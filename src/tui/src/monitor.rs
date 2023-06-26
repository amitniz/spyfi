use core::time;
use std::collections::HashMap;
use std::sync::mpsc::{Sender,Receiver};
use wpa::NetworkInfo;
use aux::{IPCMessage,IPC,IOCommand};

const MAX_CHANNEL: usize = 11;
pub type MonitorSender = Sender<IPCMessage<HashMap<String,NetworkInfo>>>;
pub type MonitorReciever = Receiver<IPCMessage<HashMap<String,NetworkInfo>>>;

pub struct MonitorThread{
    iface: String,
    channels: IPC<HashMap<String,NetworkInfo>>,
    networks: HashMap<String,NetworkInfo>,
    sweep_mode: bool,
}

impl MonitorThread{
    pub fn init(iface:&str, rx:MonitorReciever, tx:MonitorSender) -> Self{
        MonitorThread{
            iface: iface.to_owned(),
            channels: IPC{
                rx,
                tx
            },
            networks: HashMap::new(),
            sweep_mode: false,
        }

    }

    pub fn run(&mut self){
        let mut channel = 0;
        loop{
            if self.sweep_mode{
                channel = aux::modulos((channel+1) as i32,(MAX_CHANNEL+1) as i32) as u8;
                wlan::switch_iface_channel(&self.iface, channel);
            }
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
            self.channels.tx.send(IPCMessage::Message(self.networks.clone()));
            //check if got EndCommunication message
            if let Ok(msg) = self.channels.rx.try_recv(){
                if let IPCMessage::EndCommunication = msg{
                    return;
                }else if let IPCMessage::IOCommand(cmd)  = msg{
                    match cmd{
                        IOCommand::Sweep =>{
                            self.sweep_mode = true;
                        },
                        IOCommand::ChangeChannel(c) =>{
                            wlan::switch_iface_channel(&self.iface, c);
                            self.sweep_mode = false;
                        },
                        _ =>{}//do nothing
                    }
                }
            }
        }
    }

}



