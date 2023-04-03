use std::{collections::HashSet};
use pnet_datalink::{self, Channel, Config, NetworkInterface};
use libwifi::Frame;
use wlan;

const MAX_CHANNEL: u8 = 18;

/* TODOS:
*  - channel sweeping.
*  - print ssids from different channels.
*  - print clients of a network.
*  - deauth.
*/


pub fn switch_to_monitor(iface: &str) {
    wlan::toggle_power(&iface, false).unwrap();
    wlan::toggle_monitor_mode(&iface, true).unwrap();
    wlan::toggle_power(&iface, true).unwrap()
}

fn get_interface(iface: &str) -> Option<NetworkInterface>{
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces.iter().find(|i|i.name == iface);
    interface.cloned()
}

pub fn list_interfaces(){
    println!("[+] Available Interfaces:");
    for iface in pnet_datalink::interfaces(){
        println!(" * {}",iface.name);
    }
}

pub fn iface_info(iface: &str)-> std::io::Result<()>{
    let iface = get_interface(iface)
        .ok_or(std::io::Error::last_os_error())?;
    println!("{}",iface);
    Ok(())    
}

pub fn list_networks_old(iface: &str){
    //get interface
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces.into_iter().find(|i| i.name == iface).unwrap();
    // set promiscuous
    let mut config = Config::default();
    config.promiscuous = true;
    // get a read channel to the interface
    let channel = pnet_datalink::channel(&interface, config).unwrap();
    
    if let Channel::Ethernet(_, mut rx) = channel {
        let mut ssids: HashSet<String> = HashSet::default();
        while true {
            let frame = rx.next().unwrap();
            let pkt = libwifi::parse_frame(&frame[36..]);
            if pkt.is_err() {
                continue;
            }
            match pkt.unwrap() {
                Frame::Beacon(beacon) => {
                    ssids.insert(beacon.station_info.ssid.unwrap());
                }
                Frame::ProbeRequest(req) => {
                    ssids.insert(req.station_info.ssid.unwrap());
                }
                Frame::ProbeResponse(res) => {
                    ssids.insert(res.station_info.ssid.unwrap());
                }
                _ => {}
            }
            print!("Found: ");
            for ssid in &ssids {
                print!(" {}", ssid);
            }
            print!("\r");
        }
    }
}



pub fn list_networks(iface: &str) -> std::io::Result<()>{
    //get interface
    let iface = get_interface(iface)
        .ok_or(std::io::Error::last_os_error())?;

    //get a channel to the interface
    let mut config = Config{
        promiscuous: true,
        ..Config::default()
    };
    let rx = pnet_datalink::channel(&iface,config)?;
    
    //scan networks while channel sweeping
    let mut iface_channel:u8  = 0;
    let mut ssids: HashSet<String>;
    while iface_channel <=MAX_CHANNEL{


    }

    println!("{}",iface);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

}
