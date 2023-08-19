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
    Attack(AttackInfo),
    IOCommand(IOCommand),
    PermissionsError,
    EndCommunication,
}

pub struct IPC<T>{
    pub rx: Receiver<IPCMessage<T>>,
    pub tx: Sender<IPCMessage<T>>,
}

