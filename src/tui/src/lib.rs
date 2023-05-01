use std::{io,error::Error};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Rect, Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    //create app
    let res = app(&mut terminal);

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


fn app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()>{
    loop {
        terminal.draw(|f| ui(f))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
    .split(Rect {
        x: 2,
        y: 2,
        width: 10,
        height: 10,
    });

    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
}
