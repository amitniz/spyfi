//! # aux
//! `aux` is a collection of utilities to help performing certain
//!  calculations and tasks more conveniently.

// ---------------------------- Aux Functions ---------------------------------

use std::io::Write;

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

// converts signal strength into signal icon
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

//iterator implementation
struct Generator<T, F>
where
    F: FnMut(&T) -> T,T:Clone
{
    state: T,
    generator_fn: F,
}

impl<T, F> Generator<T, F>
where
    F: FnMut(&T) -> T,T: Clone
{
    fn new(initial_state: T, generator_fn: F) -> Self {
        Generator {
            state: initial_state,
            generator_fn,
        }
    }
}

impl<T, F> Iterator for Generator<T, F>
where
    F: FnMut(&T) -> T, T: Clone
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let tmp = self.state.clone();
        self.state = (self.generator_fn)(&self.state);
        Some(tmp)
    }
}

pub struct PhoneNumbers{
    prefix: String,
    generator: Generator<usize,Box< dyn FnMut(&usize) -> usize>>
}

impl PhoneNumbers{
    pub fn new(prefix:&str) -> Self{
        PhoneNumbers{
            prefix: prefix.to_owned(),
            generator: Generator::new(0,Box::new(|s:&usize|s+1)),
        }
    }

    pub fn size(&self) -> usize{
        usize::pow(10,(10 - self.prefix.len() as u32))
    }


}

impl Iterator for PhoneNumbers{
    type Item = String;

    fn next(&mut self) -> Option<String>{
        let res = self.generator.next()?;
        if res >= self.size(){
            return None;
        }
        let padding = 10 -self.prefix.len().min(10);
        if padding  == 0{
           Some(self.prefix.clone())
        }else{
            Some(format!("{}{:0>p$}",self.prefix,res, p=padding))
        }
    }
}


pub fn debug_log(fmt:std::fmt::Arguments) -> std::io::Result<()>{
    let mut f = std::fs::File::options()
        .append(true)
        .create(true)
        .open("spyfi.log")?;
    f.write_fmt(fmt);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_generator() {
        let mut adder = Generator::new(0,Box::new(|s:&usize|s+1));
        assert_eq!(0, adder.next().unwrap());
        assert_eq!(1, adder.next().unwrap());
        assert_eq!(2, adder.next().unwrap());
    }

    #[test]
    fn test_phones_generator(){
        let mut phones = PhoneNumbers::new("054");
        assert_eq!("0540000000",&phones.next().unwrap());
        assert_eq!("0540000001",&phones.next().unwrap());
        assert_eq!("0540000002",&phones.next().unwrap());
        for _ in 0..100{
            phones.next().unwrap();
        }
        assert_eq!("0540000103",&phones.next().unwrap());
    }
}
