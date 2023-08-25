use std::thread::Thread;

use crate::Handshake;

const MAX_THREADS: u8 = 150;


//TODO
#[derive(Debug,Clone)]
pub struct AttackInfo{
    pub hs: Handshake,
    pub wordlist: String,
    pub num_of_threads: u8,
    pub previous_attempts: Vec<String>,
    pub size_of_wordlist: usize,
    pub num_of_attempts: usize,
    pub network_password: Option<String>,
    pub is_attacking: bool,
    pub input_selection: InputSelection,
}


impl AttackInfo{
    
    pub fn new(handshake: Handshake,wordlist: &str,threads: u8) -> Self{
        AttackInfo{
            hs: handshake,
            wordlist: wordlist.to_owned(),
            num_of_threads: threads,
            size_of_wordlist:0,
            num_of_attempts:0,
            previous_attempts: vec![],
            network_password: None,
            is_attacking: false,
            input_selection: InputSelection::Wordlist,
        }
    }

    pub fn update(&mut self,size_of_wordlist:usize,num_of_attempts:usize,
                                            previous_attempts:Vec<String>){
        if self.size_of_wordlist == 0{
            self.size_of_wordlist = size_of_wordlist;
        }
        self.num_of_attempts = num_of_attempts;
        self.previous_attempts = [self.previous_attempts.clone(),previous_attempts].concat();
    }

    pub fn attack(&mut self){
        self.is_attacking = true;
    }

    pub fn abort(&mut self){
        self.is_attacking = false;
    }

    pub fn get_num_of_threads(self) -> u8{
        self.num_of_threads
    }

    pub fn get_hs(self) -> Handshake {
        self.hs.clone()
    }

    pub fn set_threads(&mut self,threads:u8){
        self.num_of_threads = match threads> MAX_THREADS{
            true => MAX_THREADS,
            false => threads
        };
    }
    
    pub fn set_password(&mut self,password:&str){
        self.network_password = Some(password.to_owned());
    }

    pub fn cracked(&self) -> Option<String>{
        self.network_password.clone()
    }

    pub fn progress(&self) -> u16{
        match self.size_of_wordlist{
            0=> 0,
            _=> (100*self.num_of_attempts/self.size_of_wordlist) as u16
        }
    }

    pub fn is_attacking(&self) -> bool{
        self.is_attacking
    }

    pub fn get_input_selection(&self)-> &str{
        match self.input_selection{
            InputSelection::Wordlist => "wordlist",
            InputSelection::Threads => "threads"
        }
    }

    pub fn change_selection(&mut self){
        match self.input_selection{
            InputSelection::Wordlist => self.input_selection = InputSelection::Threads,
            InputSelection::Threads  => self.input_selection =InputSelection::Wordlist
        }
    }
}


// stores the input selection for the user wordlist/threads
#[derive(Debug,Copy,Clone)]
pub enum InputSelection{
    Wordlist,
    Threads,
}
