//! # crypto
//! `crypto` contains all the key generators and cryptographic algorithms.
use pbkdf2::{pbkdf2_hmac,pbkdf2_hmac_array};
use sha1::Sha1;

const ITERATIONS: u32 = 4096;

/// Generate PSK
/// ## Description
/// Pre-Shared Key is a key generated using a password and a SSID 
/// string during handshake process. It is a 256 bit key that is calculated 
/// with pbkdf2 algorithm that uses Sha1.
/// ## Example
/// **Basic usage:**
/// ```
///     let x = crypto::generate_psk("12345678", "test");
///     let answer: [u8;32] = fe727aa8b64ac9b3f54c72432da14faed933ea511ecab15bbc6c52e7522f709a; // FIXME
///     assert_eq!(answer, x);
/// ```
pub fn generate_psk(passphrase:&str,ssid:&str) -> [u8;32]{
    pbkdf2_hmac_array::<Sha1, 32>(passphrase.as_bytes(),ssid.as_bytes(),ITERATIONS)
}



