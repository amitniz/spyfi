use std::ffi::CString;
use std::str::FromStr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


pub fn switch_channel(iface:&str, channel:u8) -> std::io::Result<()>{
    unsafe{
        let iface_cstr = CString::new(iface).unwrap();
        if c_switch_channel(iface_cstr.as_ptr() as *mut i8, channel as u32) !=0{
            Err(std::io::Error::last_os_error())
        }else{
            Ok(())
        }
    }
}

pub fn get_channel(iface: &str) -> std::io::Result<u8>{
    unsafe{
        let iface_cstr = CString::new(iface).unwrap();
        let channel = c_get_channel(iface_cstr.as_ptr() as *mut i8);
        if channel == -1 {
            Err(std::io::Error::last_os_error())
        }else{
            Ok(channel as u8)
        }
    }
}

pub fn toggle_power(iface:&str,state:bool) -> std::io::Result<()>{
    unsafe{
        let iface_cstr = CString::new(iface).unwrap();
        if c_toggle_power(iface_cstr.as_ptr() as *mut i8, state) !=0{
            Err(std::io::Error::last_os_error())
        }else{
            Ok(())
        }
    }
}

pub fn toggle_monitor_mode(iface:&str,state:bool)-> std::io::Result<()>{
    unsafe{
        let iface_cstr = CString::new(iface).unwrap();
        if c_toggle_monitor_mode(iface_cstr.as_ptr() as *mut i8, state) !=0{
            Err(std::io::Error::last_os_error())
        }else{
            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_bindings() {
        unsafe{
            //toggle off
            assert_eq!(c_toggle_power("wlan1".as_ptr() as *mut i8,false),0);
            //toggle on
            assert_eq!(c_toggle_power("wlan1".as_ptr() as *mut i8,true),0);
        }
    }
    #[test]
    fn check_power() {
        //toggle off
        toggle_power("wlan1",false).unwrap();
        //toggle on
        toggle_power("wlan1",true).unwrap();
    }

    #[test]
    fn switch_to_monitor(){
        //toggle off
        toggle_power("wlan1",false).unwrap();
        //turn monitor on
        toggle_monitor_mode("wlan1", true).unwrap();
        //toggle on
        toggle_power("wlan1",true).unwrap();
    }
}
