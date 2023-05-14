//! # cli
//! `cli` is a command line tool to control SpyFi. It allows you to 
//!  find nearest networks and perform actions such as disconnect them 
//! from the WiFi.

use clap::{Parser, ValueEnum};
use wlan;
use wpa;
use crypto;

use hex::encode;

const MAX_CHANNEL: usize = 11;
const PACKET_PER_CHANNEL: usize = 20; //num of packets to read per channel while sweeping

/// contains the arguments of the CLI
/// ## Description
/// The struct contains the arguments of the CLI and description of the commands
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// name of the wlan interface
    #[arg(short, long, group = "interface")]
    iface: Option<String>,
    
    ///generate psk 
    #[arg(long,requires= "ssid_group")]
    psk: Option<String>,

    //TODO: fix deauth group requirements
    /// send deauth message to disconnect a client
    #[arg(short, long, group ="deauth_group", requires="deauth_group")]
    deauth: bool,

    //TODO: add validation
    /// bssid of the network (i.e AABBCCDDEEFF)
    #[arg(short, long)]
    bssid: Option<String>,

    /// mac address of the target (if None, disconnect all)
    #[arg(short, long,requires = "deauth_group")]
    target: Option<String>,

    #[arg(short, long, group = "ssid_group")]
    ssid: Option<String>,

    #[arg(long, requires = "ssid_group")]
    handshake: bool,
    /// list available interfaces
    #[arg(short, long)]
    list: bool,

    /// set the interface mode
    #[arg(value_enum, requires = "interface")]
    mode: Option<Mode>,

    /// print interface info
    #[arg(long, requires = "interface")]
    info: bool,

    /// set capturing channel
    #[arg(short, long, requires = "interface", value_parser = clap::value_parser!(u8).range(0..18))]
    channel: Option<u8>,

    #[arg(long, requires = "interface")]
    networks: bool,
}

/// Enum with all modes of the interface
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    ///managed mode
    Managed,
    ///monitor mode
    Monitor,
}

/// The function that runs the tool
/// ## Description
/// Run the program with the command `sudo ./spyfi` and the  
/// arguments according to the desired action.
/// ## Example
/// **Basic usage:**
/// - Command that presents networks the interface finds:
/// 
///     `sudo ./spyfi -i wlan0 --networks`
/// - Command that changes the interface to Monitor mode:
/// 
///     `sudo ./spyfi -i wlan0 -mode Monitor`
/// - Command that changes the channel of the interface to channel 5:
/// 
///     `sudo ./spyfi -i wlan0 -c 5`
/// - Command that presents the list of the available interfaces:
/// 
///     `sudo ./spyfi -l`
pub fn run() {
    let args = Args::parse(); //parse arguments

    // mode arg
    if let Some(mode) = args.mode {
        let iface = args.iface.as_ref().unwrap();
        match mode {
            Mode::Managed => {
                if let Err(err) = wlan::toggle_monitor_state(iface, false) {
                    println!("Error: {}", err);
                }
                println!("{} switched to managed mode", iface);
            }
            Mode::Monitor => {
                if let Err(err) = wlan::toggle_monitor_state(iface, true) {
                    println!("Error: {}", err);
                };

                println!("{} switched to monitor mode", iface);
            }
        }
    }

    // channel arg
    if let Some(channel) = args.channel {
        wlan::switch_iface_channel(args.iface.as_ref().unwrap(), channel).unwrap();
        println!("switched to channel {}", channel);
    }

    // list arg
    if args.list {
        wlan::list_interfaces();
    }

    //info arg
    if args.info {
        wlan::iface_info(args.iface.as_ref().unwrap());
    }
    
    //deauth TODO:update
    if args.deauth{
        let target = args.target.as_deref().unwrap_or("broadcast");
        println!("sending deauth to {target} from bssid: {}",args.bssid.as_ref().unwrap());
        for _ in 1..64{
            wpa::send_deauth(args.iface.as_ref().unwrap(),args.bssid.as_ref().unwrap(),args.target.clone());
        }
    }

    //networks arg
    if args.networks {
        let mut channel: usize = 1;
        let mut networks : [Vec<wpa::NetworkInfo>; MAX_CHANNEL] = Default::default();

        let mut last_print_lines = 1; //TODO: for the print, move to Aux
        while channel <= MAX_CHANNEL{
            match wpa::list_channel_networks(args.iface.as_ref().unwrap(), channel as u8, PACKET_PER_CHANNEL){
                Ok(network_list) =>{
                    networks[channel-1] = network_list;
                }
                Err(err) =>{
                    todo!();
                }
            }
            /* 
            //pretty print
            //hide cursor
            println!("\x1b[25l");
            //move up the cursor
            println!("\x1b[{}A", last_print_lines);
            //print ssids
            println!("\x1b[1;30;37mSSIDS:\x1b[0m");
            println!("{:-<80}", "");
            for s in ssids.iter() {
                println!("\x1b[2K\x1b[1;32m{}\x1b[0m", s);
            }
            println!("{:-<80}", "");
            //update line count
            last_print_lines = networks.0.len() + 5;
            */
            //channel sweeping
            
            for i in 0..MAX_CHANNEL{
                for network in &networks[i]{
                    println!("{}\n\n", network);
                }
            }

            channel = (aux::modulos(channel as i32, MAX_CHANNEL as i32) as u8 + 1) as usize;
        
        }

    }

    if let Some(psk) = args.psk{
        println!("{}",encode(crypto::generate_psk(psk.as_ref(),args.ssid.as_ref().unwrap())))
    }

    if args.handshake {
        let hs = wpa::get_hs(args.iface.as_ref().unwrap(), args.ssid.as_ref().unwrap(),args.bssid.as_ref().unwrap()).unwrap();
        println!("Got HandShake!\n----------\n{}",hs);
    }
}
