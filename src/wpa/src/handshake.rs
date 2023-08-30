use crate::consts::*; //import crate defines
use std::time::{SystemTime, UNIX_EPOCH};
use aux::debug_log;
use libwifi::Addresses;
use core::fmt;

use crypto;

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
    pub fn new(ssid: &str, hs_pkts:[Option<EapolMsg>;4]) -> Handshake{
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


    pub fn get_bssid(self) ->String{
        hex::encode(self.station_mac)
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
