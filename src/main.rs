use std::collections::HashSet;
use pnet_datalink::{self, Channel, Config};
use libwifi::Frame;
use clap::{Parser,ValueEnum};
use wlan;


fn switch_to_monitor(iface: &str) {
    wlan::toggle_power(&iface, false).unwrap();
    wlan::toggle_monitor_mode(&iface, true).unwrap();
    wlan::toggle_power(&iface, true).unwrap();
}

fn scan_networks(iface: &str){
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces.into_iter().find(|i| i.name == iface).unwrap();
    let mut config = Config::default();
    config.promiscuous = true;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// name of the wlan interface
    #[arg(short, long, required = true)]
    iface: String,

    /// set the interface mode
    #[arg(value_enum)]
    mode: Option<Mode>,

    /// set capturing channel
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..18))]
    channel: Option<u8> 
}

#[derive(Copy, Clone,Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    ///managed mode
    Managed,
    ///monitor mode
    Monitor
}


fn main() {
    let args = Args::parse(); //parse arguments

    if let Some(mode) = args.mode{
        match mode{
            Mode::Managed =>{
                wlan::toggle_power(&args.iface, false).unwrap();
                println!("{} switched to managed mode",&args.iface);
            },
            Mode::Monitor =>{
                wlan::toggle_power(&args.iface, true).unwrap();
                println!("{} switched to monitor mode",&args.iface);
            }
        }
    }

    if let Some(channel) = args.channel{
        wlan::switch_channel(&args.iface, channel).unwrap();
        println!("switched to channel {}", channel);
    }
}
