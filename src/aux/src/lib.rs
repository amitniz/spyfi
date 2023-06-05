//! # aux
//! `aux` is a collection of utilities to help performing certain
//!  calculations and tasks more conveniently.


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

//check if two equal sized array are the same
//TODO: description + tests
pub fn compare_arrays<const N:usize>(a: &[u8;N],b:&[u8;N]) -> bool{
    for i in 0..N{
        if a[i] != b[i]{
            return false;
        }
    }
    return true;
}

