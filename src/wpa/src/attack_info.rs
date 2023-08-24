use crate::Handshake;





//TODO
#[derive(Debug,Clone)]
pub struct AttackInfo{
    pub hs: Handshake,
    pub wordlist: String,
    pub num_of_threads: u8,
    pub progress: u8,
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
            progress: 0,
            size_of_wordlist:0,
            num_of_attempts:0,
            previous_attempts: vec![],
            network_password: "".to_owned(),
            is_attacking: false,
        }
    }

    pub fn update(&mut self){todo!()}

    pub fn attack(&mut self){
        self.is_attacking = true;
    }

    pub fn abort(&mut self){
        self.is_attacking = false;
    }

    pub fn is_attacking(&self) -> bool{
        self.is_attacking
    }
}
