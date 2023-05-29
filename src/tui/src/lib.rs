use std::{io::{self, Stdout},error::Error, default};
use rand::Rng;
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
use screens::{Screen,colorscheme};

pub struct Tui{
    screen: Box<dyn Screen<CrosstermBackend<Stdout>>>,// the current screen
    program_state:ProgramState,
    theme: colorscheme::Theme,
}

impl Tui{

    pub fn new() -> Self{
        Tui{
            screen:Box::new(screens::WelcomeScreen::default()),
            program_state: ProgramState::default(),
            theme: colorscheme::Theme::default(),
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
                if !self.screen.handle_input(key){
                    // if the key wasn't handled by the screen
                    // GLOBAL SHORTKEYS
                    match key.code{
                        
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            res = Ok(());
                            break;
                        }
                        KeyCode::Char('p') => {self.randomize_theme() }
                        _ => {}
                    }

                }
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

    pub fn randomize_theme(&mut self){
        let mut rng = rand::thread_rng();
        let rand_int = rng.gen_range(1..6);
        match rand_int{
            1=>{self.theme = colorscheme::Theme::eggplant();}
            2=>{self.theme = colorscheme::Theme::jamaica();}
            3=>{self.theme = colorscheme::Theme::megaman();}
            4=>{self.theme = colorscheme::Theme::desert();}
            5=>{self.theme = colorscheme::Theme::pokemon();}
            6=>{self.theme = colorscheme::Theme::default();}
            _=>{}
        }
        //prevent from choosing the current theme
        if self.screen.theme_name() == self.theme.name{
            self.randomize_theme()
        }else{
            self.screen.set_theme(self.theme.clone());
        }
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

