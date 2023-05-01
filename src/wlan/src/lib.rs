use std::ffi::CString;

use pnet_datalink::{self, Channel, Config, DataLinkReceiver,DataLinkSender, NetworkInterface};
use std::io::{Error, ErrorKind};
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));



pub fn list_interfaces() {
    println!("[+] Available Interfaces:");
    for iface in pnet_datalink::interfaces() {
        println!(" * {}", iface.name);
    }
}

pub fn iface_info(iface: &str) -> std::io::Result<()> {
    let iface = get_interface(iface).ok_or(std::io::Error::last_os_error())?;
    println!("{}", iface);
    Ok(())
}

pub fn get_send_channel(iface: &str) -> std::io::Result<Box<dyn DataLinkSender>> {

    // get interface
    let iface = get_interface(iface).ok_or(Error::last_os_error())?;

    // get a channel to the interface
    let config = Config {
        promiscuous: true,
        read_timeout: Some(std::time::Duration::from_millis(50)),
        ..Config::default()
    };
    
    let channel = pnet_datalink::channel(&iface, config)?;
    if let Channel::Ethernet(tx,_) = channel {
        Ok(tx)
    } else {
        Err(Error::new(ErrorKind::Other, "unknown error"))
    }
}


pub fn get_recv_channel(iface: &str) -> std::io::Result<Box<dyn DataLinkReceiver>> {
    // get interface
    let iface = get_interface(iface).ok_or(Error::last_os_error())?;

    // get a channel to the interface
    let config = Config {
        promiscuous: true,
        read_timeout: Some(std::time::Duration::from_millis(50)),
        ..Config::default()
    };
    
    let channel = pnet_datalink::channel(&iface, config)?;
    if let Channel::Ethernet(_, rx) = channel {
        Ok(rx)
    } else {
        Err(Error::new(ErrorKind::Other, "unknown error"))
    }
}

pub fn switch_iface_channel(iface: &str, channel: u8) -> std::io::Result<()> {
    unsafe {
        let iface_cstr = CString::new(iface).unwrap();
        if c_switch_channel(iface_cstr.as_ptr() as *mut i8, channel as u32) != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}



//TODO: the c code is broken
pub fn get_iface_channel(iface: &str) -> std::io::Result<u8> {
    unsafe {
        let iface_cstr = CString::new(iface).unwrap();
        let channel = c_get_channel(iface_cstr.as_ptr() as *mut i8);
        if channel == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(channel as u8)
        }
    }
}

pub fn toggle_power(iface: &str, state: bool) -> std::io::Result<()> {
    unsafe {
        let iface_cstr = CString::new(iface).unwrap();
        if c_toggle_power(iface_cstr.as_ptr() as *mut i8, state) != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub fn toggle_monitor_mode(iface: &str, state: bool) -> std::io::Result<()> {
    unsafe {
        let iface_cstr = CString::new(iface).unwrap();
        if c_toggle_monitor_mode(iface_cstr.as_ptr() as *mut i8, state) != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub fn get_interface(iface: &str) -> Option<NetworkInterface> {
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces.iter().find(|i| i.name == iface);
    interface.cloned()
}

pub fn toggle_monitor_state(iface: &str, mode: bool) -> std::io::Result<()> {
    toggle_power(&iface, false)?;
    toggle_monitor_mode(&iface, mode)?;
    toggle_power(&iface, true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_bindings() {
        unsafe {
            //toggle off
            assert_eq!(c_toggle_power("wlan1".as_ptr() as *mut i8, false), 0);
            //toggle on
            assert_eq!(c_toggle_power("wlan1".as_ptr() as *mut i8, true), 0);
        }
    }
    #[test]
    fn check_power() {
        //toggle off
        toggle_power("wlan1", false).unwrap();
        //toggle on
        toggle_power("wlan1", true).unwrap();
    }

    #[test]
    fn switch_to_monitor() {
        //toggle off
        toggle_power("wlan1", false).unwrap();
        //turn monitor on
        toggle_monitor_mode("wlan1", true).unwrap();
        //toggle on
        toggle_power("wlan1", true).unwrap();
    }
}
