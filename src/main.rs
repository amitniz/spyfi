use clap::{Parser,ValueEnum};
use aux;
use wlan;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// name of the wlan interface
    #[arg(short, long, group="interface")]
    iface: Option<String>,
   
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
    
    // mode arg
    if let Some(mode) = args.mode{
        let iface = args.iface.as_ref().unwrap();
        match mode{
            Mode::Managed =>{
                wlan::toggle_power(iface, false).unwrap();
                println!("{} switched to managed mode",iface);
            },
            Mode::Monitor =>{
                wlan::toggle_power(iface, true).unwrap();
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
}
