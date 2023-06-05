//! # aux
//! `aux` is a collection of utilities to help performing certain
//!  calculations and tasks more conveniently.

/* TODOS:
*  - channel sweeping.
*  - print ssids from different channels.
*  - print clients of a network.
*  - deauth.
*/

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
pub fn is_equal<const N:usize>(a: &[u8;N],b:&[u8;N]) -> bool{
    for i in 0..N{
        if a[i] != b[i]{
            return false;
        }
    }
    return true;
}

pub mod screen{
    use core::fmt;
    use std::io::{self,Write};

    pub enum Style{
        Normal,
        Bold,
        Italic,
        Faint,
        Underline,
        SlowBlink,
        RapidBlink,
        Invert,
        Strike,
        PrimaryFont,
        AlternativeFont,
    }

    impl fmt::Display for Style{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let code;
            match self{
                Style::Normal=>{ code = 0; }
                Style::Bold =>{ code = 1; }
                Style::Faint =>{ code = 2; }
                Style::Italic =>{ code = 3; }
                Style::Underline =>{ code = 4; }
                Style::SlowBlink =>{ code = 5; }
                Style::RapidBlink =>{ code = 6; }
                Style::Invert =>{ code = 7; }
                Style::Strike =>{ code = 9; }
                Style::PrimaryFont =>{ code = 10; }
                Style::AlternativeFont =>{ code = 11; }
            }
            write!(f,"{code}")
        }
    }

    pub enum Color{
        Black ,
        Red ,
        Green ,
        Yellow ,
        Blue ,
        Magenta ,
        Cyan ,
        White ,
        BrightBlack ,
        BrightRed ,
        BrightGreen ,
        BrightYellow ,
        BrightBlue ,
        BrightMagenta ,
        BrightCyan ,
        BrightWhite ,
    }

    impl fmt::Display for Color{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let code;
            match self{
                Color::Black =>{ code = 30; }
                Color::Red =>{ code = 31; }
                Color::Green =>{ code = 32; }
                Color::Yellow =>{ code = 33; }
                Color::Blue =>{ code = 34; }
                Color::Magenta =>{ code = 35; }
                Color::Cyan =>{ code = 36; }
                Color::White =>{ code = 37; }
                Color::BrightBlack =>{ code = 90; }
                Color::BrightRed =>{ code = 91; }
                Color::BrightGreen =>{ code = 92; }
                Color::BrightYellow =>{ code = 93; }
                Color::BrightBlue =>{ code = 94; }
                Color::BrightMagenta =>{ code = 95; }
                Color::BrightCyan =>{ code = 96; }
                Color::BrightWhite =>{ code = 97; }
            }
            write!(f,"{code}")
        }
    }
    /// hides the cursor
    pub fn hide_cursor(){
        print!("\x1b[25l");
        io::stdout().flush();
    }

    /// moves up the cursor N lines
    pub fn move_up_cursor(n_lines:usize){
        print!("\x1b[{n_lines}A");
        io::stdout().flush();
    }
   

    /// returns a styled str
    #[macro_export]
    macro_rules! style {
        ($s:expr, $a:ident) =>{
            {
                if type($a) != Color || type($a) != Style{
                    compile_error!("style! can only get Color or Style as arguments");
                }
                format!("{}{}\x1b[0m",$s,$a)
            }
        };
 
        ($text:literal, $color:item,$style:item,$($rest:ident),*) =>{
            {
                if type($color) != Color || type($color) != Style{
                    compile_error!("style! can only get Color or Style as arguments");
                }

                if type($style) != Style{
                    compile_error!("style! can only get Style as not first arguments");
                } 

                format!("{style!($text,$(rest),*)}")
            }
        };
    }
}


// -------------------------------- Tests -------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        println!(style!("text",Color:Red));
    }
}
