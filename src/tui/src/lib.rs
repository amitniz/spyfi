mod screens;
mod items;
mod monitor;

use std::{
    io::{self, Stdout},
    error::Error,
    sync::{RwLock,mpsc},
    thread, collections::HashMap,
};
use aux::{IPCMessage, IOCommand};
use rand::Rng;
use lazy_static::lazy_static;
use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode
    },
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode, 
        EnterAlternateScreen, 
        LeaveAlternateScreen
    },
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};


use screens::{Screen,colorscheme};
use monitor::MonitorThread;
use wpa::NetworkInfo;


// ---------------------------------- Macros ----------------------------------
#[macro_export]
macro_rules! create_list {
    ($inst: expr,$title:literal,$list:expr) => {
        //TODO: add type check to item (should be Vec<String>)
        List::new($list.iter().map(|i|{ListItem::new(format!(" 🍕 {} ",i))}).collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title($title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg($inst.theme.text).bg($inst.theme.border_bg))
            )
            .style(Style::default().bg($inst.theme.bg).fg($inst.theme.text))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg($inst.theme.bright_text).bg($inst.theme.highlight))
    }
}

// -------------------------------- Static ------------------------------------
lazy_static! {
    static ref GLOBAL_CONFIGURATIONS: GlobalConfigs = GlobalConfigs::default();
}


// -------------------------------- Structs -----------------------------------
pub struct Tui{
    screen: Box<dyn Screen<CrosstermBackend<Stdout>>>,// the current screen
    ipc_channels: Option<aux::IPC<HashMap<String,NetworkInfo>>>, //IPC channels for communicating with monitor thread
}

impl Tui{

    pub fn new() -> Self{
        Tui{
            screen:Box::new(screens::WelcomeScreen::default()),
            ipc_channels: None,
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
                        aux::IPCMessage::Message(netinfo) =>{
                            //update screen data
                            let res = self.screen.update(aux::IPCMessage::Message(netinfo));
                            if let  Some(msg) = res{
                                ipc.tx.send(msg);
                            }
                        },
                        aux::IPCMessage::PermissionsError =>{
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
        self.ipc_channels = Some(aux::IPC{
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
            ipc.tx.send(aux::IPCMessage::EndCommunication);
        }
    }

    fn randomize_theme(&mut self){
        let mut rng = rand::thread_rng();
        let rand_int = rng.gen_range(1..3);
        let theme;
        match rand_int{
            1=>{theme = colorscheme::Theme::eggplant();}
            2=>{theme = colorscheme::Theme::desert();}
            _=>{theme = colorscheme::Theme::default();}
        }
        //prevent from choosing the current theme
        if GlobalConfigs::get_instance().get_theme_name() == theme.name{
            self.randomize_theme()
        }else{
            self.screen.set_theme(&theme);
            GlobalConfigs::get_instance().set_theme(&theme);

        }
    }

}

/// Storing information of 
/// different states such as the choosen .interface, his channel and mode.
#[derive(Default)]
struct GlobalConfigs{
    iface: RwLock<String>,
    channel: RwLock<String>,
    mode: RwLock<String>,
    theme: RwLock<colorscheme::Theme>,
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

    pub fn get_mode(&self) -> String{
        self.mode.read().unwrap().clone()
    }

    pub fn set_mode(&self, mode:&str) {
        *self.mode.write().unwrap() = mode.to_owned();
    }

    pub fn get_channel(&self) -> String{
        self.channel.read().unwrap().clone()
    }

    pub fn set_channel(&self, channel: &str){
        *self.channel.write().unwrap() = channel.to_owned();
    }
    pub fn get_theme_name(&self) -> String{
        self.theme.read().unwrap().name.clone()
    }

    pub fn set_theme(&self,theme: &colorscheme::Theme){
        *self.theme.write().unwrap() = theme.clone();
    }
}




