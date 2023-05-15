use pbkdf2::{pbkdf2_hmac,pbkdf2_hmac_array};
use sha1::Sha1;
use hmac::{Hmac, Mac};
use concat_arrays::concat_arrays;
use hex::FromHex;

const ITERATIONS: u32 = 4096;


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

pub fn mic_data(data:&[u8;121]) -> [u8;121]{
    let pre_data:[u8;81] = data[..81].try_into().unwrap();
    let post_data:[u8;24] = data[97..].try_into().unwrap();
    concat_arrays!(pre_data,[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],post_data)
}


//TODO: return result instead of unwraping
pub fn digest_hmac_sha1<const K:usize,const N:usize>(key: &[u8;K],msg: &[u8;N]) -> [u8;20]{
        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(key).unwrap();
        mac.update(msg);
        mac.finalize().into_bytes().into()
}

// generate PTK
// TODO: 
// make sure to digest every iteration with reseting the mac. dont use update for each iteration.
// return an array of the key
pub fn generate_ptk(psk: &[u8;32], client_mac: &[u8;6], station_mac:&[u8;6], a_nonce:&[u8;32], s_nonce:&[u8;32]) -> [u8;64]{
    let b: [u8;76] = concat_arrays!(min(client_mac,station_mac),max(client_mac,station_mac),min(a_nonce,s_nonce),max(a_nonce,s_nonce));
    prf_512(psk,b"Pairwise key expansion", b)
}

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
