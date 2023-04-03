use clap::{Parser,ValueEnum};
use aux;
use wlan;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// name of the wlan interface
    #[arg(short, long, group="interface")]
    iface: Option<String>,
   
    #[arg(short, long, requires = "interface", group="ssid_group")]
    ssid: Option<String>,
 
    #[arg(long,requires="ssid_group")]
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

#[derive(Copy, Clone,Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    ///managed mode
    Managed,
    ///monitor mode
    Monitor
}

pub fn run(){
    let args = Args::parse(); //parse arguments

    // mode arg
    if let Some(mode) = args.mode{
        let iface = args.iface.as_ref().unwrap(); 
        match mode{
            Mode::Managed =>{
                if let Err(err) = aux::toggle_monitor_state(iface, false){
                    println!("Error: {}",err);
                }
                println!("{} switched to managed mode",iface);
            },
            Mode::Monitor =>{
                if let Err(err) = aux::toggle_monitor_state(iface, true){
                    println!("Error: {}",err);
                };

                println!("{} switched to monitor mode",iface);
            }
        }
    }


    // channel arg
    if let Some(channel) = args.channel{

        wlan::switch_channel(args.iface.as_ref().unwrap(), channel).unwrap();
        println!("switched to channel {}", channel);
    }

    // list arg
    if args.list {
        aux::list_interfaces();
    }

    //info arg
    if args.info {
       aux::iface_info(args.iface.as_ref().unwrap());
    }

    if args.networks{
        if let Err(error) = aux::list_networks(args.iface.as_ref().unwrap()){
            println!("Error: {}",error);
        }
    }

    if args.handshake{
        aux::get_hs(args.iface.as_ref().unwrap(),args.ssid.as_ref().unwrap()).unwrap();
    }
}