use std::{io::{self, Stdout},error::Error, default};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

mod screens;
use screens::Screen;

 pub struct Tui{
    screen: Box<dyn Screen<CrosstermBackend<Stdout>>>,// the current screen
    program_state:ProgramState,
}

impl Tui{

    pub fn new() -> Self{
        Tui{
            screen:Box::new(screens::WelcomeScreen::default()),
            program_state: ProgramState::default(),
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        //create app
        let res: io::Result<()>;
        loop {
            terminal.draw(|f| self.screen.set_layout(f))?;

            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    res = Ok(());
                    break;
                }
                self.screen.handle_input(key);
            }
        }

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res{
            println!("{:?}",err);
        }

        Ok(())
    }
}

#[derive(Default)]
/// Storing information of 
/// different states such as the choosen interface, his channel and mode.
struct ProgramState{
     iface: Option<String>,
     channel: Option<String>,
     mode: Option<String>,
}

