/* TODOS:
*  - channel sweeping.
*  - print ssids from different channels.
*  - print clients of a network.
*  - deauth.
*/

// ---------------------------- Aux Functions ---------------------------------

pub fn modulos(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
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
                Normal=>{ code = 0; }
                Bold =>{ code = 1; }
                Faint =>{ code = 2; }
                Italic =>{ code = 3; }
                Underline =>{ code = 4; }
                SlowBlink =>{ code = 5; }
                RapidBlink =>{ code = 6; }
                Invert =>{ code = 7; }
                Strike =>{ code = 9; }
                PrimaryFont =>{ code = 10; }
                AlternativeFont =>{ code = 11; }
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
                Black =>{ code = 30; }
                Red =>{ code = 31; }
                Green =>{ code = 32; }
                Yellow =>{ code = 33; }
                Blue =>{ code = 34; }
                Magenta =>{ code = 35; }
                Cyan =>{ code = 36; }
                White =>{ code = 37; }
                BrightBlack =>{ code = 90; }
                BrightRed =>{ code = 91; }
                BrightGreen =>{ code = 92; }
                BrightYellow =>{ code = 93; }
                BrightBlue =>{ code = 94; }
                BrightMagenta =>{ code = 95; }
                BrightCyan =>{ code = 96; }
                BrightWhite =>{ code = 97; }
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
