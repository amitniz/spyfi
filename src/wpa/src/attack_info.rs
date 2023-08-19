

#[derive(Debug,Clone)]
pub enum AttackInfo{
    DictionaryAttack(DictionaryAttack),
    DeauthAttack(DeauthAttack),
}


#[derive(Debug,Clone,Default)]
pub struct DeauthAttack{
    pub bssid: String,
    pub client: Option<String>,
    pub station_channel: u8,
}


#[derive(Debug,Clone)]
pub struct DictionaryAttack{
    pub wordlist: String,
    pub num_of_threads: u8,
    pub progress: u8,
    pub previous_attempts: Vec<String>,
    pub network_password: String,
    is_attacking: bool,
}

impl Default for DictionaryAttack{
    fn default() -> Self {
        DictionaryAttack{
            wordlist: "".to_owned(),
            num_of_threads: 1,
            progress: 0,
            previous_attempts: vec![],
            network_password: "".to_owned(),
            is_attacking: false,
        }
    }
}


impl DictionaryAttack{

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




