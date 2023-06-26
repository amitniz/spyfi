//! # aux
//! `aux` is a collection of utilities to help performing certain
//!  calculations and tasks more conveniently.

use std::sync::mpsc::{Sender,Receiver};
// ---------------------------- Aux Functions ---------------------------------

/// Calculates modulo between two given numbers
/// ## Description
/// Modulo operation returns the reminder or signed reminder of a division,
/// after one number is divided by another.
/// ## Example
/// **Basic usage:**
/// ```
///     let x = aux::modulos(12, 5);
///     println!("12 mod 5 is: {}", x);
///     assert_eq!(2, x);
/// ```
pub fn modulos(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

/// Check if two equal sized array are the same
/// ## Description
/// Receives 2 arrays of the same size and returns whether the arrays are 
/// equal by comparing all elements.
/// ## Example
/// **Basic usage:**
/// ```
///     let a = [10, 4, 8];
///     let b = [10, 4, 8];
///     let c = [20, 6, 8];
/// 
///     let mut equal = aux::is_equal(&a, &b);
///     assert_eq!(true, equal);
///     equal = aux::is_equal(&a, &c);
///     assert_eq!(false, equal);
/// ```
pub fn compare_arrays<const N:usize>(a: &[u8;N],b:&[u8;N]) -> bool{
    for i in 0..N{
        if a[i] != b[i]{
            return false;
        }
    }
    return true;
}

pub fn signal_icon(signal_strength: i8) -> String{
    let signals = ["󰤟","󰤢","󰤨"];
    if signal_strength > -50{
        signals[2].to_owned()
    }else if  signal_strength > -70{
        signals[1].to_owned()
    }else{
        signals[0].to_owned()
    }
}
#[derive(Clone)]
pub enum IOCommand{
    Sweep,
    ChangeChannel(u8),
    SendDeauth,
}

#[derive(Clone)]
pub enum IPCMessage<T>{
    Message(T),
    Password(String),
    IOCommand(IOCommand),
    PermissionsError,
    EndCommunication,
}

pub struct IPC<T>{
    pub rx: Receiver<IPCMessage<T>>,
    pub tx: Sender<IPCMessage<T>>,
}

