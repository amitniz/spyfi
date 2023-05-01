use tui;
use cli;
use aux::{self,screen};


// entrypoint
fn main() {
    if std::env::args().len() == 1{
        //tui::run();
        // let color = format!({},screen::Color::Red);
        // let styled_text = aux::style!("test",color);
        // println!("{}",color);

    }else{
        cli::run();
    }
}
