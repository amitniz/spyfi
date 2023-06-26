//! # cli
// ! `cli` is a command line tool to control SpyFi. It allows you to 
//!  find nearest networks and perform actions such as disconnect them 
//! from the WiFi.

use std::{io::{BufRead,BufReader}, collections::HashMap};
use std::{sync::mpsc,thread};
use clap::{Parser, ValueEnum, Args, Subcommand};
use wlan;
use aux::{IPC,IPCMessage};
use wpa::{self, NetworkInfo};
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
    /// listen for nearby stations or clients
    #[arg(short,long,group="enumerate")]
    listen: ListenMode,

    #[arg(short,long,id="BPF Filter",group="enumerate",requires="outputfile")]
    capture: Option<String>,

    /// name of the wlan interface
    #[arg(short, long)]
    iface: String,
    
    #[arg(short,long, required_if_eq("listen","clients"))]
    ssid: Option<String>,

    ///timeout in seconds
    #[arg(short, long,default_value_t=60)]
    timeout: u32,

    ///dump results into an outputfile
    #[arg(short,long,requires="enumerate")]
    outputfile: Option<String>,

    ///use channel sweeping
    #[arg(long,requires="enumerate")]
    sweep:bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ListenMode {
    Stations,
    Clients,
}

impl Enumerate{
    fn parse(&self){
        //if let Some(bpf_filter) = self.capture.as_ref(){
        //    todo!("not implemented");
        //}

        match self.listen{
            ListenMode::Stations =>{
                let iface = self.iface.as_ref();
                let interval = std::time::Duration::from_millis(INTERVAL);
                let timeout = std::time::Duration::from_secs(self.timeout as u64);
                let init_time = std::time::Instant::now();
                let mut channel: u8 = 1;
                let mut networks:HashMap<String,NetworkInfo> = HashMap::new();
                while init_time.elapsed() <= timeout {
                    if self.sweep{
                        wlan::switch_iface_channel(iface, channel);
                        channel = aux::modulos((channel+1) as i32,(MAX_CHANNEL+1) as i32) as u8;
                    }
                    let stations = wpa::listen_and_collect(iface,interval);
                    match stations {
                        Ok(stations) =>{
                            for (_,mut station) in stations{
                                networks.entry(station.ssid.clone())
                                    .and_modify(|e|  e.update(&mut station))
                                    .or_insert(station);
                            } 
                        },
                        Err(e) =>{
                            todo!("handle errors");
                        }
                    }
                    networks_pretty_print(&networks);
                }
            },
            ListenMode::Clients =>{
                unimplemented!();
            }
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
    #[arg(long,short,group="source")]
    iface: Option<String>,
   
    ///capture file
    #[arg(short,long,group="source")]
    capture: Option<String>,

    ///number of threads to use for dictionary/bruteforce attack
    #[arg(short,long,default_value_t=1,value_parser = clap::value_parser!(u32).range(1..201))]
    threads:u32,

    ///regex pattern for bruteforce
    //#[arg(long)]
    //pattern:Option<String>,

    ///send deauths to achieve handshake faster (NOISY)
    #[arg(long,required_if_eq("ATTACKTYPE","bruteforce"))]
    aggresive:bool,

    ///target MAC for DoS attack [default: broadcast]
    #[arg(long)]
    target: Option<String>,

    ///network's SSID. Might be used instead of BSSID and we will try to get it (Unrecommended)
    #[arg(
        short, 
        long,
        required_if_eq("ATTACKTYPE","dict"),
        required_if_eq("ATTACKTYPE","bruteforce"),
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
    #[arg(long,group="dict")]
    wordlist: Option<String>,
    
    ///phonenumbers generator
    #[arg(long,group="dict")]
    phones: bool,
}


#[derive(ValueEnum, Debug, Ord, Eq, PartialOrd, Clone, PartialEq)]
enum AttackType{
    ///dictionary file
    Dict, 
    /// Deauth attack
    Dos,
    /// Bruteforce attack
    Bruteforce,
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
                            hs = wpa::get_hs_from_file(cap, ssid, bssid);

                        },
                        _ =>{
                            println!("failed to open pcap:{}",self.capture.as_ref().unwrap());
                            return;
                        } 
                    }        
                   
                    let mut threads:Vec<IPC<String>> = vec![];
                    // create threads pool
                    for _ in 0..self.threads{
                        let (thread_tx,main_rx) = mpsc::channel();
                        let (main_tx,thread_rx) = mpsc::channel(); 

                        let main_ipc:IPC<String> = IPC{
                            tx: main_tx,
                            rx: main_rx,
                        };

                        let thread_ipc:IPC<String> = IPC{
                            tx: thread_tx,
                            rx: thread_rx,
                        };
                        
                        threads.push(main_ipc);
                        let hs_cpy = hs.as_ref().unwrap().clone();
                        thread::spawn(move||{
                            wpa::password_worker(thread_ipc,hs_cpy);
                        });
                    }
                    
                    let mut jobs_count = 0;
                    let mut next_thread: i32 = 0;
                    //TODO:make it the right way
                    if self.phones{

                        let prefixes = vec!["050","052","053","054","057"];
                        for prefix in &prefixes{
                            for i in 0..10e7 as i32 {
                                let passwd = format!("{}{:0>7}",prefix,i).to_owned();
                                jobs_count += 1;
                                threads[next_thread as usize].tx.send(IPCMessage::Message(passwd.to_owned()));
                                next_thread = aux::modulos(next_thread+1, threads.len() as i32);
                            }
                        }

                    }else{

                        // read passwords from dictionary
                        let wordlist = self.wordlist.as_ref().unwrap();
                        let path = std::fs::File::open(&self.wordlist.as_ref().unwrap()).unwrap();
                        let reader = BufReader::new(path);
                        
                        
                        for line in reader.lines(){
                            let line = match line.as_ref(){
                                Ok(pass) => {
                                    if pass.len() < 8{
                                        println!("[-] invalid size: {}",pass);
                                        continue;
                                    }
                                    pass
                                },
                                Err(_) => {continue}
                            };
                            jobs_count += 1;
                            threads[next_thread as usize].tx.send(IPCMessage::Message(line.to_owned()));
                            next_thread = aux::modulos(next_thread+1, threads.len() as i32);
                        }
                    }


                    while jobs_count > 0{
                        for thread in &threads{
                            if let Ok(ipc_msg) = thread.rx.try_recv(){
                                match ipc_msg{
                                    IPCMessage::Message(msg) =>{
                                        match msg.as_str(){
                                            "wrong" =>{
                                                jobs_count -= 1;
                                            },
                                            _ =>{
                                                println!("[+] Found password: {}",msg);
                                                //kill all threads
                                                for thread in &threads{
                                                    thread.tx.send(IPCMessage::EndCommunication);
                                                }
                                                return;
                                            }
                                        }
                                    },
                                    _ =>{
                                        continue;
                                    }
                                }
                            }        
                        }
                    }
                    println!("[!] Exhausted.")
                }


            },
            AttackType::Bruteforce =>{

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
            println!("Not implemented yet");
        }
        
    }
}

// ----------------------------------- Aux ------------------------------------


fn networks_pretty_print(networks:&HashMap<String,NetworkInfo>){
    //hide cursor
    println!("\x1b[25l");
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
            println!("{:^15}|",hex::encode(station.clients[0]));
            for client in &station.clients[1..]{
                println!("{:>67}  |",hex::encode(client));
            }
            lines_num += station.clients.len()-1;
        }else{
            println!("               |");
        }
    }
    println!(" --------------------------------------------------------------------");
    //move up the cursor
    print!("\x1b[{}A",lines_num);
}
