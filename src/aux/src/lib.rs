#[warn(unused_variables)]
use std::{collections::HashSet};
use std::io::{Error,ErrorKind};
use pnet_datalink::{self, Channel,DataLinkReceiver, Config, NetworkInterface};
use libwifi::Frame;
use wlan;

const MAX_CHANNEL: u8 = 11;

/* TODOS:
*  - channel sweeping.
*  - print ssids from different channels.
*  - print clients of a network.
*  - deauth.
*/


// --------------------------------- Structs ----------------------------------

//TODO: use in list networks
struct Network{
    station_addr: [u8;6],
    ssid: String,
    channel: u8,
    strength: i8,
}


// ---------------------------- Aux Functions ---------------------------------


fn modulos(a: i32,b: i32) -> i32{
    ((a % b) + b) % b
}

fn get_interface(iface: &str) -> Option<NetworkInterface>{
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces.iter().find(|i|i.name == iface);
    interface.cloned()
}

fn get_recv_channel(iface:&str) -> std::io::Result<Box<dyn DataLinkReceiver>>{
    // get interface
    let iface = get_interface(iface)
        .ok_or(Error::last_os_error())?;

    // get a channel to the interface
    let mut config = Config{
        promiscuous: true,
        read_timeout: Some(std::time::Duration::from_millis(50)),
        ..Config::default()
    };
    let channel = pnet_datalink::channel(&iface,config)?;
    if let Channel::Ethernet(_,mut rx) = channel{
        Ok(rx) 
    }else{
        Err(Error::new(ErrorKind::Other,"unknown error"))
    }
    
}

// ------------------------- Public Functions ---------------------------------

pub fn toggle_monitor_state(iface: &str,mode: bool) -> std::io::Result<()> {
    wlan::toggle_power(&iface, false)?;
    wlan::toggle_monitor_mode(&iface, mode)?;
    wlan::toggle_power(&iface, true)?;
    Ok(())
}


pub fn list_interfaces(){
    println!("[+] Available Interfaces:");
    for iface in pnet_datalink::interfaces(){
        println!(" * {}",iface.name);
    }
}

//TODO: change to get_hs(iface: &str,ssid:&str,channel:u8) -> std::io::Result<vec<Network>>
// and remove the while
pub fn get_hs(iface:&str,ssid:&str) ->std::io::Result<()>{
    let mut rx = get_recv_channel(iface)?;
    let mut iface_channel:u8 = 4;
    while iface_channel <= MAX_CHANNEL{
        //set channel
        wlan::switch_channel(iface,iface_channel)?;
        for i in 0..10{
            let mut frame;
            match rx.next(){
                Ok(data) =>{
                    frame = data;
                },
                _ => {continue;} //timeout
            }
            let pkt = libwifi::parse_frame(&frame[36..]);
            if pkt.is_err(){
                continue; //TODO: what gets here
            }
            if let Frame::QosData(qos) = pkt.unwrap() {
                let data = qos.data;
                // make sure that is key 
                let msg_type:u16 = ((data[6] as u16) << 8) | data[7] as u16;
                if msg_type == 0x888e{ //Auth type
                    println!("[EAPOL] data:\n{:02x?}",data);
                }
            }

        }
        // channel sweeping
        //iface_channel = modulos(iface_channel as i32,MAX_CHANNEL as i32) as u8 + 1;
    }
    Ok(())
}

pub fn iface_info(iface: &str)-> std::io::Result<()>{
    let iface = get_interface(iface)
        .ok_or(std::io::Error::last_os_error())?;
    println!("{}",iface);
    Ok(())    
}


pub fn list_clients(iface:&str, ssid: &str) -> std::io::Result<()>{
    todo!();
}


//TODO: change to list_networks(iface: &str,channel:u8) -> std::io::Result<vec<Network>>
// and remove the while
pub fn list_networks(iface: &str) -> std::io::Result<()>{
    // scan networks while channel sweeping
    let mut iface_channel:u8  = 1;
    let mut ssids: HashSet<String> = Default::default();
    // get rx channel to the given interface
    let mut rx = get_recv_channel(iface)?; 
    while iface_channel <= MAX_CHANNEL{
        // set channel
        wlan::switch_channel(iface,iface_channel)?;
        for i in 0..10{ //TODO: consider improving
            let mut frame;
            match rx.next(){
                Ok(data) =>{
                    frame = data;
                },
                _ => {continue;} //timeout
            }
            let signal = frame[30] as i8;
            let pkt = libwifi::parse_frame(&frame[36..]);
            if pkt.is_err(){
                continue;
            }
            if let Frame::Beacon(beacon) = pkt.unwrap() {
                //println!("{:?}",&beacon);
                if let Some(ssid) = beacon.station_info.ssid{
                    if !ssids.contains(&format!("{} {}",ssid.clone(), iface_channel)){
                        ssids.insert(format!("{} {}",ssid.clone(), iface_channel));
                        println!("[+] ssid: {} [channel {} signal {}dBm]",ssid,iface_channel,signal);
                    }
                }
            }
        }
        // channel sweeping
        iface_channel = modulos(iface_channel as i32,MAX_CHANNEL as i32) as u8 + 1;
    }
    Ok(())
}

// -------------------------------- Tests -------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

}
