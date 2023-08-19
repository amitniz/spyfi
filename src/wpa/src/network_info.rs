use crate::{
    handshake::{Handshake, EapolMsg},
    attack_info::AttackInfo,
    consts::*, DictionaryAttack
};
use std::{convert::TryFrom, collections::HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use libwifi::Addresses;
use core::fmt;


type Client = String;

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
                    frame_type: None,
                    attack_info: None ,
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
    pub frame_type: Option<FrameType>,
    pub attack_info: Option<DictionaryAttack>,
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

impl NetworkInfo{
    pub fn update(&mut self,other: &mut NetworkInfo){

        // update the channel to be the channel of the bssid handshakes
        match &other.frame_type.as_ref().unwrap(){
            FrameType::Beacon | FrameType::AssociationRequest | FrameType::AssociationResponse => self.channel = other.channel,
            _=>{},
        };
        self.signal_strength = other.signal_strength;
        self.clients.append(other.clients.as_mut());
        //remove duplications
        self.clients.sort();
        self.clients.dedup();
        self.last_appearance = other.last_appearance;

        if self.ssid == "unknown"{
            self.ssid = other.ssid.clone();
        }
        
        if self.protocol == "unknown"{
            self.protocol = other.protocol.clone();
        }
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
                ssid:"unknown".to_owned(),
                bssid,
                channel: None,
                signal_strength:None,
                clients,
                handshake: None,
                captured_handshakes: HashMap::new(),
                protocol: "unknown".to_owned(),
                last_appearance: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                frame_type: None,
                attack_info: None,
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

#[derive(Debug,Clone)]
pub enum FrameType{
    Beacon,
    Data,
    QosData,
    QosNull,
    ProbeRequest,
    ProbeResponse,
    AssociationRequest,
    AssociationResponse,
    Unknown,
}

//parsed frame
pub enum ParsedFrame{
    Network(NetworkInfo),
    Eapol(EapolMsg)
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
