use std::{
    io::{self, BufReader,BufRead, Read, Write},
    fs,
    sync::mpsc::{channel,Sender,Receiver},
    thread,
};
use crate::ipc::{IPC,IPCMessage,AttackMsg, AttackProgress};
use aux::debug_log;
use wpa::{Handshake,AttackInfo};


const JOB_SIZE: usize = 36;
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

    pub fn run(&mut self) -> io::Result<()>{
        //spawn threads
        let mut threads: Vec<JobIPC> = vec![];
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


        for thread in &threads{
            if generator_mode{
                let iterator = phones.as_mut().unwrap().by_ref().take(JOB_SIZE);
                self.send_job_to_worker(thread,iterator);
            }else{
                let iterator = readbuf_to_iter(reader.as_mut().unwrap().by_ref());
                self.send_job_to_worker(thread,iterator);
            }
        }

        loop{
            if let Ok(msg) = self.ipc_channels.rx.try_recv(){
                if let IPCMessage::Attack(AttackMsg::Abort) = msg{
                    break;
                }
            } 
            //read msgs from threads
            for thread in &threads{
                if let Ok(job) = thread.rx.try_recv(){
                    match job{
                        Job::Found(password) => {
                            self.ipc_channels.tx.send(IPCMessage::Attack(AttackMsg::Password(password)));
                           //TODO: break didn't allow the main thread to read it before it closed. close thread by message from the thread
                        },
                        Job::Done =>{
                            if generator_mode{
                                let iterator = phones.as_mut().unwrap().by_ref().take(JOB_SIZE);//readbuf_to_iter(reader.by_ref());
                                self.send_job_to_worker(thread,iterator);
                            }else{
                                let iterator = readbuf_to_iter(reader.as_mut().unwrap().by_ref());
                                self.send_job_to_worker(thread,iterator);
                            }
                        },
                        _ => {},
                    }
                }
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
