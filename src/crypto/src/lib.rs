//! # crypto
//! `crypto` contains all the key generators and cryptographic algorithms.
use pbkdf2::{pbkdf2_hmac,pbkdf2_hmac_array};
use sha1::Sha1;
use hmac::{Hmac, Mac};
use concat_arrays::concat_arrays;
use hex::FromHex;

const ITERATIONS: u32 = 4096;

/// Generate PSK
/// ## Description
/// Pre-Shared Key is a key generated using a password and a SSID 
/// string during handshake process. It is a 256 bit key that is calculated 
/// with pbkdf2 algorithm that uses Sha1.
/// 
/// The function receives 2 strings that represent the password and the SSID
/// and returns an array of the PSK.
/// ## Example
/// **Basic usage:**
/// ```
///     let x = crypto::generate_psk("12345678", "test");
///     let answer: [u8;32] = [0xfe, 0x72, 0x7a, 0xa8, 0xb6, 0x4a, 0xc9, 0xb3,
///                            0xf5, 0x4c, 0x72, 0x43, 0x2d, 0xa1, 0x4f, 0xae,
///                            0xd9, 0x33, 0xea, 0x51, 0x1e, 0xca, 0xb1, 0x5b,
///                            0xbc, 0x6c, 0x52, 0xe7, 0x52, 0x2f, 0x70, 0x9a];
///     assert_eq!(answer, x);
/// ```
pub fn generate_psk(passphrase:&str,ssid:&str) -> [u8;32]{
    pbkdf2_hmac_array::<Sha1, 32>(passphrase.as_bytes(),ssid.as_bytes(),ITERATIONS)
}

fn min<const N: usize>(a:&[u8; N],b:&[u8; N]) -> [u8;N] {
    for i in 0..N {
        if a[i] < b[i]{
            return a.clone();
        }
        else if a[i] != b[i]{
            break;
        }
    }
    b.clone()
}

fn max<const N: usize>(a:&[u8; N],b:&[u8; N]) -> [u8;N] {
    for i in 0..N {
        if a[i] > b[i]{
            return a.clone();
        }
        else if a[i] != b[i] {
           break; 
        }
    }
    b.clone()
}

/// Collect the relevant MIC data to one array
/// ## Description
/// Receives the data message of a frame and 
/// reassemble the relevant data to a MIC message.
pub fn mic_data(data:&[u8;121]) -> [u8;121]{
    let pre_data:[u8;81] = data[..81].try_into().unwrap();
    let post_data:[u8;24] = data[97..].try_into().unwrap();
    concat_arrays!(pre_data,[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],post_data)
}


/// Digest of a message using HMAC Sha1
/// ## Description
/// Return digest of a message for given secret key and digest.
/// 
/// The function receives a key and a message and uses HMAC Sha1 
/// to return an array of the digest of a message.
//TODO: return result instead of unwraping
pub fn digest_hmac_sha1<const K:usize,const N:usize>(key: &[u8;K],msg: &[u8;N]) -> [u8;20]{
        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(key).unwrap();
        mac.update(msg);
        mac.finalize().into_bytes().into()
}

/// Generate PTK
/// ## Description
/// Pairwise Transient key is used to encrypt all unicast traffic between a 
/// client station and the access point. PTK is unique between a client station 
/// and access point. To generate PTK, the following information is required:
/// 
/// PTK = PRF-512(PSK + MAC1 + MAC2 + Anonce + SNonce)
/// 
/// * PSK - higher-level key computed with password and SSID
/// * MAC(AA)- the mac address of the AP (authenticator).
/// * MAC(SA)- the mac address of the client (supplicant).
/// * MAC1 - min between MAC(AA) and MAC(SA)
/// * MAC2 - max between MAC(AA) and MAC(SA)
/// * ANONCE- is a random number that the AP has made.
/// * SNONCE- is a random number that the client has made.
///
/// ## Example
/// **Basic usage:**
/// ```
///     let SSID = "Praneeth";
///     let password = "kemparajanusha";
///     let psk = crypto::generate_psk(password, SSID);
///     let client_mac = [0xc0, 0xf4, 0xe6, 0x4b, 0x6a, 0xcf];
///     let station_mac = [0x60, 0xe3, 0x27, 0xf8, 0x14, 0xa0];
///     let a_nonce = [0xac, 0x98, 0x71, 0xc9, 0xca, 0x12, 0x94, 0x68,
///                    0x70, 0x8c, 0xa0, 0xd5, 0x54, 0xe2, 0x2f, 0x4f,
///                    0x8b, 0x6e, 0xaa, 0x6d, 0xba, 0xa1, 0x21, 0xd2,
///                    0x23, 0x3b, 0xf3, 0x3c, 0xbc, 0x29, 0xd3, 0x46];
///     let s_nonce = [0x52, 0x14, 0xc4, 0xdb, 0xe4, 0xa5, 0x67, 0xe7,
///                    0x8b, 0x8f, 0x30, 0xb2, 0xb0, 0x16, 0xa2, 0xd9,
///                    0x0e, 0xa5, 0x0c, 0x27, 0xd4, 0x08, 0x61, 0x4c,
///                    0x1f, 0xc0, 0xa0, 0x93, 0x4a, 0x88, 0x9a, 0xda];
/// 
///     let ptk = crypto::generate_ptk(&psk, &client_mac, &station_mac, &a_nonce, &s_nonce);
/// 
///     let answer: [u8;64] = [0xfb, 0x18, 0x56, 0x0e, 0x63, 0x90, 0x9f, 0x84,
///                            0xf3, 0x1d, 0x39, 0xda, 0x03, 0xa5, 0xd8, 0x2f,
///                            0xdc, 0x78, 0xc3, 0xb5, 0x6f, 0x18, 0x70, 0x54,
///                            0x43, 0x08, 0xb8, 0x4d, 0xee, 0x21, 0x44, 0xb8,
///                            0x76, 0x15, 0x72, 0x9c, 0x48, 0x84, 0xa5, 0x45,
///                            0xd3, 0x92, 0xc2, 0x0b, 0x3f, 0x69, 0x70, 0x25,
///                            0x63, 0x32, 0x45, 0xfc, 0x5a, 0x0f, 0xa1, 0x5e,
///                            0xfe, 0xb0, 0xc8, 0x25, 0x01, 0xf3, 0xa7, 0xb4];
///     assert_eq!(answer, ptk);
/// ```
// TODO: 
// make sure to digest every iteration with reseting the mac. dont use update for each iteration.
// return an array of the key
pub fn generate_ptk(psk: &[u8;32], client_mac: &[u8;6], station_mac:&[u8;6], a_nonce:&[u8;32], s_nonce:&[u8;32]) -> [u8;64]{
    let b: [u8;76] = concat_arrays!(min(client_mac,station_mac),max(client_mac,station_mac),min(a_nonce,s_nonce),max(a_nonce,s_nonce));
    prf_512(psk,b"Pairwise key expansion", b)
}


/// Generate KCK
/// ## Description
/// Key Confirmation Key (KCK) is the first 16 bytes of the PTK.
/// It is used to compute MIC for integrity.To generate KCK, 
/// the following information is required:
/// 
/// KCK = PRF-128(PSK + MAC1 + MAC2 + Anonce + SNonce)
/// 
/// * PSK - higher-level key computed with password and SSID
/// * MAC(AA)- the mac address of the AP (authenticator).
/// * MAC(SA)- the mac address of the client (supplicant).
/// * MAC1 - min between MAC(AA) and MAC(SA)
/// * MAC2 - max between MAC(AA) and MAC(SA)
/// * ANONCE- is a random number that the AP has made.
/// * SNONCE- is a random number that the client has made.
///
/// ## Example
/// **Basic usage:**
/// ```
///     let SSID = "Praneeth";
///     let password = "kemparajanusha";
///     let psk = crypto::generate_psk(password, SSID);
///     let client_mac = [0xc0, 0xf4, 0xe6, 0x4b, 0x6a, 0xcf];
///     let station_mac = [0x60, 0xe3, 0x27, 0xf8, 0x14, 0xa0];
///     let a_nonce = [0xac, 0x98, 0x71, 0xc9, 0xca, 0x12, 0x94, 0x68,
///                    0x70, 0x8c, 0xa0, 0xd5, 0x54, 0xe2, 0x2f, 0x4f,
///                    0x8b, 0x6e, 0xaa, 0x6d, 0xba, 0xa1, 0x21, 0xd2,
///                    0x23, 0x3b, 0xf3, 0x3c, 0xbc, 0x29, 0xd3, 0x46];
///     let s_nonce = [0x52, 0x14, 0xc4, 0xdb, 0xe4, 0xa5, 0x67, 0xe7,
///                    0x8b, 0x8f, 0x30, 0xb2, 0xb0, 0x16, 0xa2, 0xd9,
///                    0x0e, 0xa5, 0x0c, 0x27, 0xd4, 0x08, 0x61, 0x4c,
///                    0x1f, 0xc0, 0xa0, 0x93, 0x4a, 0x88, 0x9a, 0xda];
/// 
///     let kck = crypto::generate_kck(&psk, &client_mac, &station_mac, &a_nonce, &s_nonce);
/// 
///     let answer: [u8;16] = [0xfb, 0x18, 0x56, 0x0e, 0x63, 0x90, 0x9f, 0x84,
///                            0xf3, 0x1d, 0x39, 0xda, 0x03, 0xa5, 0xd8, 0x2f];
///     assert_eq!(answer, kck);
/// ```
// TODO: 
// make sure to digest every iteration with reseting the mac. dont use update for each iteration.
// return an array of the key
pub fn generate_kck(psk: &[u8;32], client_mac: &[u8;6], station_mac:&[u8;6], a_nonce:&[u8;32], s_nonce:&[u8;32]) -> [u8;16]{
    let b: [u8;76] = concat_arrays!(min(client_mac,station_mac),max(client_mac,station_mac),min(a_nonce,s_nonce),max(a_nonce,s_nonce));
    prf_128(psk,b"Pairwise key expansion", b)
}


/// PRF-512 to compute PTK
/// ## Description
/// Pseudo-Random Function that is used in WPA to generate the PTK.
/// The function incorporating a text string into the input and 
/// designed to produce a certain number of bits, in this case 512.
/// 
/// The function requiers the following information:
/// 
/// * k - Random number
/// * a - The text string
/// * b - A sequence of bytes formed by the MAC address 
//TODO: remove unwrap
fn prf_512(k:&[u8;32] ,a: &[u8;22], b:[u8;76]) -> [u8;64]{
    let mut key:Vec<[u8;20]> = vec!(); 
    for i in 0..4{
        let msg:[u8;100] = concat_arrays!(a.clone(),[0],b,[i]);
        key.push(digest_hmac_sha1(&k,&msg));
    }

    let ptk:[u8;80] = concat_arrays!(key[0],key[1],key[2],key[3]);
    ptk[0..64].try_into().unwrap()
}

/// PRF-128 to compute KCK
/// ## Description
/// Pseudo-Random Function that is used in WPA to generate the PTK.
/// The function incorporating a text string into the input and 
/// designed to produce a certain number of bits, in this case 128.
/// 
/// The function requiers the following information:
/// 
/// * k - Random number
/// * a - The text string
/// * b - A sequence of bytes formed by the MAC address 
//TODO: remove unwrap
fn prf_128(k:&[u8;32] ,a: &[u8;22], b:[u8;76]) -> [u8;16]{
    let msg:[u8;100] = concat_arrays!(a.clone(),[0],b,[0]);
    let key:[u8;20] = digest_hmac_sha1(&k,&msg);
    key[0..16].try_into().unwrap()
}

#[cfg(test)]
mod tests{
    use super::*;
    use hex::ToHex;
    #[test]
    fn test_concatings_arrays(){
        let res: [u8;12] = concat_arrays!([11,22,33,44,55,66],[22,33,44,55,66,77]);
        assert_eq!(res,[11,22,33,44,55,66,22,33,44,55,66,77]);
        println!("[+] passed check 1: concat([11,22,33,44,55,66],[22,33,44,55,66,77])");

        assert_eq!(concat_arrays!([11,22,33,44,55,66],[22,33,44,55,66,77],[33,44,55,66,77,88]),[11,22,33,44,55,66,22,33,44,55,66,77,33,44,55,66,77,88]);
        println!("[+] passed check 2: concat([11,22,33,44,55,66],[22,33,44,55,66,77],[33,44,55,66,77,88])"); 
    }
    
    #[test]
    fn test_min_max_arrays(){
        let a:[u8;6]  = [0xa0,0x11,0xb2,0x30,0x00,0x01];
        let b:[u8;6]  = [0xa0,0x11,0xb2,0x20,0x00,0x01];
        let c:[u8;6]  = [0xa0,0x11,0xb2,0x30,0x00,0x05];
    
        let mut res:[u8;6] = min(&a,&b);
        assert_eq!(res,b);
        println!("[+] passed check 1: min(a,b) = b");
        res = max(&a,&b);
        assert_eq!(res,a);
        println!("[+] passed check 2: max(a,b) = a");

        res = min(&a,&c);
        assert_eq!(res,a);
        println!("[+] passed check 3: min(a,c) = a");

        res = min(&a,&a);
        assert_eq!(res,a);
        assert_eq!(res,max(&a,&a));
        println!("[+] passed check 4: min(a,a) = max(a,a) = a");
        
    }

    #[test]
    fn test_digest(){
        let res = digest_hmac_sha1(b"supersecretkey",b"message").encode_hex::<String>();
        assert_eq!("084825919f741a7fb18b80f1ad2610c729b52df7",res);
        println!("[+] passed check: hmac_sha1_digest('supersecretkey','message') = {}",res);
    }

}
