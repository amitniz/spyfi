// FRAME PARSING
pub const SIGNAL_POS: usize = 30;
pub const FRAME_HEADER_LENGTH: usize = 2;
pub const EAPOL_MSG_NUM_OFFSET: usize = 0xd;
pub const EAPOL_CODE_OFFSET: usize = 0x6;
pub const EAPOL_NONCE_OFFSET: usize = 0x19;
pub const EAPOL_MIC_OFFSET: usize = 0x59;

pub const EAPOL_TAG_ID: u16 = 0x888e;

// EAPOL CODES
pub const EAPOL_MSG_1: u16 = 0x8a;
pub const EAPOL_MSG_2: u16 = 0x10a;
pub const EAPOL_MSG_3: u16 = 0x13ca;
pub const EAPOL_MSG_4: u16 = 0x30a;

// WPA IDENTIFICATIONS
pub const RSN_TAG_ID: u8 = 0x30;
pub const WPA2_PSK_AKM: u8 = 0x2;
pub const WPA2_EAP_TLS_AKM: u8 = 0x1;
pub const WPA2_EAP_PEAP_AKM: u8 = 0x2;
pub const WPA2_EAP_TTLS_AKM: u8 = 0x3;
pub const WPA2_EAP_FAST_AKM: u8 = 0x4;
pub const WPA3_FT_AKM: u8 = 0x9;
pub const WPA3_SAE_AKM: u8 = 0x8;
pub const WPA3_SHA256_AKM: u8 = 0x6;
pub const AES_GROUP_CYPHER_TYPE:u8 = 0x4;
//NOTE: WPA3 needs more accuracy, i.e it can also be 0x2, so an addition condition check is needed
//for identification
