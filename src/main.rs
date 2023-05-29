use tui::Tui;
use cli;
use aux::{self,screen};


// entrypoint
fn main() {
    if std::env::args().len() == 1{
        Tui::new()
            .run();



    }else{
        cli::run();
    }
}
