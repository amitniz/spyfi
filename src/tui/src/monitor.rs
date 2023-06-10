use core::time;
use std::collections::HashMap;
use std::sync::mpsc::{Sender,Receiver};
use wpa::NetworkInfo;


pub enum IPCMessage{
    NetworkInfo(HashMap<String,NetworkInfo>),
    PermissionsError,
    EndCommunication,
}


pub struct IPC{
    pub rx: Receiver<IPCMessage>,
    pub tx: Sender<IPCMessage>,
}

pub struct MonitorThread{
    iface: String,
    channels: IPC,
    networks: HashMap<String,NetworkInfo>,
}

impl MonitorThread{
    pub fn init(iface:&str, rx:Receiver<IPCMessage>,tx:Sender<IPCMessage>) -> Self{
        MonitorThread{
            iface: iface.to_owned(),
            channels: IPC{
                rx,
                tx
            },
            networks: HashMap::new(),
        }

    }

    pub fn run(&mut self){
        loop{
            match wpa::listen_and_collect(&self.iface, time::Duration::from_secs(1)){
                Ok(networks) =>{
                    for (ssid, mut network) in networks.into_iter(){
                        self.networks.entry(ssid)
                            .and_modify(|e| e.update(&mut network))
                            .or_insert(network);
                    }
                },
                Err(_) => { self.channels.tx.send(IPCMessage::PermissionsError);},
            }
            //send back network information
            self.channels.tx.send(IPCMessage::NetworkInfo(self.networks.clone()));
            //check if got EndCommunication message
            if let Ok(msg) = self.channels.rx.try_recv(){
                if let IPCMessage::EndCommunication = msg{
                    return;
                }
            }
        }
    }

}



