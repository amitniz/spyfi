use tui::Tui;
use cli;

// entrypoint
fn main() {
    //launch tui incase no arguments provided 
    if std::env::args().len() == 1{
        Tui::new()
            .run();
    //otherwise run in cli mode
    }else{
        cli::run();
    }
}

