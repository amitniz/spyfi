use std::{
    io::{self, BufReader,BufRead, Read},
    fs,
    sync::mpsc::{channel,Sender,Receiver},
    thread, borrow::BorrowMut
};
use crate::ipc::{IPC,IPCMessage,AttackMsg, AttackProgress};
use wpa::{Handshake,AttackInfo};


const JOB_SIZE: usize = 31;

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


impl AttackThread{

    fn send_job_to_worker(&mut self, worker: &JobIPC,buf: &mut dyn BufRead){
            let passwords_list:Vec<String> = buf.lines().take(JOB_SIZE).filter_map(|line|{
                match line{
                    Ok(password) => Some(password),
                    Err(_) => None,
                }
            }).collect();
            //increment num of attempts
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
                password_worker(thread_ipc,hs);
            });
        } 
        
        //read wordlist
        let f = fs::File::open(&self.attack_info.wordlist)?;
        let mut reader = BufReader::new(f);
       
        for thread in &threads{
            self.send_job_to_worker(thread, reader.by_ref());
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
                            break;
                        },
                        Job::Done =>{
                            self.send_job_to_worker(thread, reader.by_ref());
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
