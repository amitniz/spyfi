/*
*   The file contains the code for the Attack thread.
*/
use std::{
    io::{self, BufReader,BufRead, Read, Write},
    fs,
    sync::mpsc::{channel,Sender,Receiver},
    thread,
};
use crate::ipc::{IPC,IPCMessage,AttackMsg, AttackProgress};
use wpa::{Handshake,AttackInfo};


const JOB_SIZE: usize = 37; //num of passwords per job

type AttackSender = Sender<IPCMessage<AttackMsg>>;
type AttackReciever = Receiver<IPCMessage<AttackMsg>>;

struct JobIPC{
    pub rx: Receiver<Job>,
    pub tx: Sender<Job>,
}


pub struct AttackThread{
    attack_info:AttackInfo,
    ipc_channels: IPC<AttackMsg>
}

/// readbuf_to_iter 
/// ### Description
/// creates a job sized iterator from the next bufffer lines
fn readbuf_to_iter(buf: &mut dyn BufRead) -> impl Iterator<Item = String>{
    
        buf.lines().take(JOB_SIZE).filter_map(|line|{
            match line{
                Ok(password) => Some(password),
                Err(_) => None,
            }
        }).collect::<Vec<String>>().into_iter()
}

impl AttackThread{

    fn send_job_to_worker<I>(&mut self, worker: &JobIPC,iterator:I)
        where I:Iterator<Item = String> {
            let passwords_list:Vec<String> = iterator.collect();
            //increment num of attempts
            if passwords_list.len() == 0{ return;}
            self.attack_info.num_of_attempts += passwords_list.len();
            //send to thread
            worker.tx.send(Job::Wordlist(passwords_list.clone()));
            //send back progress
            self.ipc_channels.tx.send(
                IPCMessage::Attack(AttackMsg::Progress(AttackProgress{
                    size_of_wordlist: self.attack_info.size_of_wordlist,
                    num_of_attempts: self.attack_info.num_of_attempts,
                    passwords_attempts: passwords_list
                })));
    }

    pub fn init(ipc:IPC<AttackMsg>, attack_info: AttackInfo) -> io::Result<Self>{
        Ok(AttackThread { 
            attack_info: AttackInfo{
                size_of_wordlist: Self::count_words(&attack_info.wordlist)?,
                ..attack_info
            }, 
            ipc_channels:ipc,
        })
    }
    

    /// run
    /// ### Description:
    /// contains the loop logic of the Attack thread.
    /// creates a thread pool of workers that are trying to check possible passwords. 
    pub fn run(&mut self) -> io::Result<()>{
        //spawn worker threads
        let mut threads: Vec<JobIPC> = vec![];
        let mut live_threads = self.attack_info.num_of_threads;
        for _ in 0..self.attack_info.num_of_threads{
            //create the ipc channels
            let (thread_tx,main_rx) = channel();
            let (main_tx,thread_rx) = channel(); 
           
            let thread_ipc = JobIPC{
                rx: thread_rx,
                tx: thread_tx,
            };

            //store channels
            threads.push(JobIPC{
                rx: main_rx,
                tx: main_tx,
            });
            //spawn the thread
            let hs = self.attack_info.hs.clone();
            thread::spawn(move||{
                password_worker(thread_ipc, hs);
            });
        } 
        let mut phones = None;
        let mut reader = None;
        let generator_mode: bool;
        // generator/ wordlist
        if self.attack_info.wordlist.starts_with("#phone"){
            generator_mode = true;
            let prefix = self.attack_info.wordlist.split(" ").collect::<Vec<&str>>()[1];
            phones = Some(aux::PhoneNumbers::new(prefix));
        }else{
            generator_mode = false;
            //read wordlist
            let f = fs::File::open(&self.attack_info.wordlist)?;
            reader = Some(BufReader::new(f));
        }

        //send first jobs
        for thread in &threads{
            if generator_mode{
                let mut iterator = phones.as_mut().unwrap().by_ref().take(JOB_SIZE).peekable();
                if iterator.peek().is_none(){ //incase no more jobs left
                    thread.tx.send(Job::Done);
                    live_threads -=1;
                }else{ self.send_job_to_worker(thread,iterator); }
            }else{ //wordlist
                let mut iterator = readbuf_to_iter(reader.as_mut().unwrap().by_ref()).peekable();
                if iterator.peek().is_none(){ //incase no more jobs left
                    thread.tx.send(Job::Done);
                    live_threads -=1;
                }else{ self.send_job_to_worker(thread,iterator); }
            }
        }

        loop{
            //read msg from main thread
            if let Ok(msg) = self.ipc_channels.rx.try_recv(){
                if let IPCMessage::Attack(AttackMsg::Abort) = msg{
                    break;
                }
            } 

            //read msgs from worker threads
            for thread in &threads{
                if let Ok(job) = thread.rx.try_recv(){
                    match job{
                        Job::Found(password) => {
                            self.ipc_channels.tx.send(IPCMessage::Attack(AttackMsg::Password(password)));
                            //when main thread will read the message it will send abort. 
                        },
                        Job::Done =>{
                            //TODO: find a way to make the iterator type the same for wordlist and
                            //generators to avoid this code duplication
                            if generator_mode{
                                let mut iterator = phones.as_mut().unwrap().by_ref().take(JOB_SIZE).peekable();
                                if iterator.peek().is_none(){ //incase no more jobs left
                                    thread.tx.send(Job::Done);
                                    live_threads -=1;
                                }else{ self.send_job_to_worker(thread,iterator); }
                            }else{
                                let mut iterator = readbuf_to_iter(reader.as_mut().unwrap().by_ref()).peekable();
                                if iterator.peek().is_none(){ //incase no more jobs left
                                    thread.tx.send(Job::Done);
                                    live_threads -=1;
                                }else{ self.send_job_to_worker(thread,iterator); }
                            }
                        },
                        _ => {},
                    }
                }
            }
            // check if there are still workers with job
            if live_threads <=0 {
                self.ipc_channels.tx.send(IPCMessage::Attack(AttackMsg::Exhausted));
                //when main thread will read the message it will send abort. 
            }
        }

        for thread in &threads{
            thread.tx.send(Job::Done); 
        }
        Ok(())
    }

    fn count_words(wordlist:&str) -> io::Result<usize>{
        if wordlist.starts_with("#"){
            //TODO: make in the right way
            let prefix = wordlist.split(" ").collect::<Vec<&str>>()[1];
            return Ok(usize::pow(10,10-prefix.len().min(10) as u32));
        }
        let f = fs::File::open(wordlist)?;
        let reader = BufReader::new(f);
        Ok(reader.lines().count())
    }
}

enum Job{
    Wordlist(Vec<String>), //list of passwords to try
    Found(String), ///found password
    Done, // the threads done its job or the attack has ended.
}

//A worker for cracking the network's password
fn password_worker(ipc: JobIPC,handshake: Handshake){
    loop {
        if let Ok(job_msg) = ipc.rx.recv(){
            match job_msg{
                Job::Wordlist(list) =>{
                    for password in list{
                        if handshake.clone().try_password(&password){
                            ipc.tx.send(Job::Found(password));
                            return;
                        }
                    }

                    ipc.tx.send(Job::Done);
                },
                Job::Done => {
                    return;
                },
                _ => {
                    continue;
                }
            }
        }
    }
}
