use pbkdf2::{pbkdf2_hmac,pbkdf2_hmac_array};
use sha1::Sha1;

const ITERATIONS: u32 = 4096;

pub fn generate_psk(passphrase:&str,ssid:&str) -> [u8;32]{
    pbkdf2_hmac_array::<Sha1, 32>(passphrase.as_bytes(),ssid.as_bytes(),ITERATIONS)
}



