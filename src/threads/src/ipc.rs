use std::sync::mpsc::{Sender,Receiver};
use wpa::AttackInfo;

#[derive(Clone)]
pub enum IOCommand{
    Sweep,
    ChangeChannel(u8),
}

#[derive(Clone)]
pub enum IPCMessage<T>{
    Message(T),
    Attack(AttackMsg),
    IOCommand(IOCommand),
    PermissionsError,
    EndCommunication,
}

pub struct IPC<T>{
    pub rx: Receiver<IPCMessage<T>>,
    pub tx: Sender<IPCMessage<T>>,
}


#[derive(Debug,Clone)]
pub enum AttackMsg{
    DictionaryAttack(AttackInfo),
    DeauthAttack(DeauthAttack),
    Progress(AttackProgress),
    Password(String),
    Exhausted,
    Abort,
    Error,
}


#[derive(Debug,Clone,Default)]
pub struct DeauthAttack{
    pub bssid: String,
    pub client: Option<String>,
    pub station_channel: u8,
}



#[derive(Clone,Default,Debug)]
pub struct AttackProgress{
    pub size_of_wordlist: usize, // amount of passwords in the wordlist file
    pub num_of_attempts: usize,
    pub passwords_attempts: Vec<String>,
}

