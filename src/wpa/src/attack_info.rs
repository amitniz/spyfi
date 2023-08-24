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
    pub network_password: String,
    pub is_attacking: bool,
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
            network_password: "".to_owned(),
            is_attacking: false,
        }
    }

    pub fn update(&mut self,size_of_wordlist:usize,num_of_attempts:usize,
                                            previous_attempts:Vec<String>){
        if self.size_of_wordlist == 0{
            self.size_of_wordlist = size_of_wordlist;
        }
        self.num_of_attempts = num_of_attempts;
        self.previous_attempts = previous_attempts;
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


    pub fn is_attacking(&self) -> bool{
        self.is_attacking
    }
}
