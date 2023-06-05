use std::{
    io::{self, Stdout},
    error::Error,
    sync::{RwLock,mpsc::{self, Sender, Receiver}},
    thread::{self,Thread},
};
use rand::Rng;
use lazy_static::lazy_static;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};

mod screens;
use screens::{Screen,colorscheme};
mod monitor;
use monitor::MonitorThread;

lazy_static! {
    static ref GLOBAL_CONFIGURATIONS: GlobalConfigs = GlobalConfigs::default();
}

pub struct Tui{
    screen: Box<dyn Screen<CrosstermBackend<Stdout>>>,// the current screen
    ipc_channels: Option<monitor::IPC>, //IPC channels for communicating with monitor thread
    theme: colorscheme::Theme,
}

impl Tui{

    pub fn new() -> Self{
        Tui{
            screen:Box::new(screens::WelcomeScreen::default()),
            ipc_channels: None,
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
            //get data from thread
            if let Some(ipc) = self.ipc_channels.as_ref(){
                if let Ok(msg) = ipc.rx.try_recv(){
                    match msg{
                        monitor::IPCMessage::NetworkInfo(netinfo) =>{
                            //update screen data
                            self.screen.update(monitor::IPCMessage::NetworkInfo(netinfo));
                        },
                        monitor::IPCMessage::PermissionsError =>{
                            //popup permissions screen
                            todo!("permissions error");
                        }
                        _ =>{
                            panic!();//shouldn't get here
                        }
                    }
                }
            }
            terminal.draw(|f| self.screen.set_layout(f))?;
            //read keyboard events
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.screen.handle_input(key){
                        // if the key wasn't handled by the screen
                        // GLOBAL SHORTKEYS
                        match key.code{
                            
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                res = Ok(());
                                break;
                            },
                            KeyCode::Char('p') => {self.randomize_theme() },
                            KeyCode::Enter => {
                                self.screen =Box::new(screens::MainScreen::default());
                                self.spawn_monitor_thread(); //start monitor thread
                            }
                            _ => {}
                        }

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

        self.quit();// close all threads
        Ok(())
    }

    fn spawn_monitor_thread(&mut self){
        let (thread_tx,main_rx) = mpsc::channel();
        let (main_tx,thread_rx) = mpsc::channel(); 
        self.ipc_channels = Some(monitor::IPC{
            rx: main_rx,
            tx: main_tx,
        });

        thread::spawn(move ||{
            let iface = GlobalConfigs::get_instance().get_iface();
            MonitorThread::init(&iface, thread_rx, thread_tx).run();
        });
    }

    fn quit(&self){
        if let Some(ipc) = self.ipc_channels.as_ref(){
            ipc.tx.send(monitor::IPCMessage::EndCommunication);
        }
    }


    fn randomize_theme(&mut self){
        let mut rng = rand::thread_rng();
        let rand_int = rng.gen_range(1..7);
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
struct GlobalConfigs{
     iface: RwLock<String>,
     channel: RwLock<String>,
     mode: RwLock<String>,
}


impl GlobalConfigs{
    pub fn set_iface(&self,iface:&str){
        *self.iface.write().unwrap() = iface.to_owned();
    }
    pub fn get_iface(&self) -> String{
        self.iface.read().unwrap().clone()
    }
    pub fn get_instance() -> &'static Self{
        return &GLOBAL_CONFIGURATIONS;
    }
}


