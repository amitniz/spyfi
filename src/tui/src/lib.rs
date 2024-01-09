mod screens;
mod items;

use std::{
    io::{self, Stdout},
    error::Error,
    sync::{RwLock,mpsc},
    thread, collections::HashMap,
};
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
use threads::{MonitorThread, AttackThread};
use threads::ipc::{IPC,IPCMessage,AttackMsg};
use wpa::{NetworkInfo, AttackInfo};


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
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg($inst.theme.highlight_text).bg($inst.theme.highlight))
    }
}

// -------------------------------- Static ------------------------------------
lazy_static! {
    static ref GLOBAL_CONFIGURATIONS: GlobalConfigs = GlobalConfigs::default();
}


// -------------------------------- Structs -----------------------------------
pub struct Tui{
    screen: Box<dyn Screen<CrosstermBackend<Stdout>>>,// the current screen
    monitor_ipc_channels: Option<IPC<HashMap<String,NetworkInfo>>>, //IPC channels for communicating with monitor thread
    attack_ipc_channels: Option<IPC<AttackMsg>>,
    is_panic: bool, //permissions error state
}

impl Tui{

    pub fn new() -> Self{
        Tui{
            screen:Box::new(screens::WelcomeScreen::default()),
            monitor_ipc_channels: None,
            attack_ipc_channels: None,
            is_panic: false,
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
        let mut app_res: io::Result<()> = Ok(());
        loop {
            //get data from monitor thread
            if let Some(ipc) = self.monitor_ipc_channels.as_ref(){
                if let Ok(msg) = ipc.rx.try_recv(){
                    let res = self.screen.update(msg);
                    if let Some(res) = res{
                        app_res = self.pass_screen_request(res);
                    } 
                }
            }
            //get data from attack thread
            if let Some(ipc) = self.attack_ipc_channels.as_ref(){
                if let Ok(IPCMessage::Attack(attack_msg)) = ipc.rx.try_recv(){
                    let res = self.screen.update(IPCMessage::Attack(attack_msg));
                    if let Some(req) = res{
                        app_res = self.pass_screen_request(req);
                    } 
                }
            }
            if self.is_panic{
                break;
            }
            terminal.draw(|f| self.screen.set_layout(f))?;
            //read keyboard events
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.screen.handle_input(key){
                        // if the key wasn't handled by the screen
                        // GLOBAL SHORTKEYS
                        match key.code{
                            
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                app_res = Ok(());
                                break;
                            },
                            KeyCode::Char('p') => {self.randomize_theme() },
                            KeyCode::Enter => { //choosing interface
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

        if let Err(err) = app_res{
            println!("{}",err);
        }

        self.quit();// close all threads
        Ok(())
    }

    fn spawn_attack_thread(&mut self,attack_info:AttackInfo) ->io::Result<()>{
        let (thread_tx,main_rx) = mpsc::channel();
        let (main_tx,thread_rx) = mpsc::channel(); 
        self.attack_ipc_channels = Some(IPC{
            rx: main_rx,
            tx: main_tx,
        });
        let thread_ipc = IPC{ //channels stored in the generated thread 
            rx:thread_rx,
            tx:thread_tx,
        };
        let mut attack_thread = AttackThread::init(thread_ipc,attack_info)?; //returns error incase
        //of invalid wordlist
        thread::spawn(move ||{
            attack_thread.run()
        });
        Ok(())
    }

    
    // pass ipc request of the main screen to the coresponding thread
    fn pass_screen_request(&mut self,req : IPCMessage<HashMap<String,NetworkInfo>>) ->io::Result<()>{
        match req{
            IPCMessage::Message(_) | IPCMessage::IOCommand(_)  =>{
                if let Some(ipc) = &self.monitor_ipc_channels{
                    ipc.tx.send(req);
                }
            },
            IPCMessage::PermissionsError =>{
                //toggle popup permissions screen
                self.is_panic = true;
                return Err(io::Error::new(io::ErrorKind::PermissionDenied,"Spyfi cannot run without network capabilities."));
            },
            IPCMessage::Attack(attack_msg) =>{
                match attack_msg{
                    AttackMsg::DeauthAttack(_) =>{
                        if let Some(ipc) = &self.monitor_ipc_channels{
                            ipc.tx.send(IPCMessage::Attack(attack_msg));
                        }
                    },
                    AttackMsg::DictionaryAttack(attack) =>{
                        if self.attack_ipc_channels.is_none(){//Note: For now we don't allow more than
                            //one attack at a time
                            if self.spawn_attack_thread(attack).is_err(){
                                self.screen.update(IPCMessage::Attack(AttackMsg::Error));
                            }
                        }      
                    },
                    AttackMsg::Abort =>{
                        if let Some(ipc) = &self.attack_ipc_channels{
                            // pass the attack message to the attack thread
                            ipc.tx.send(IPCMessage::Attack(attack_msg));
                            // delete the current ipc channels to the attack thread
                            self.attack_ipc_channels = None;
                        } 
                    },
                    _=>{}
                }
            },
            _=> {},
        }
        Ok(())
    }

    fn spawn_monitor_thread(&mut self){
        let (thread_tx,main_rx) = mpsc::channel();
        let (main_tx,thread_rx) = mpsc::channel(); 
        self.monitor_ipc_channels = Some(IPC{
            rx: main_rx,
            tx: main_tx,
        });

        thread::spawn(move ||{
            let iface = GlobalConfigs::get_instance().get_iface();
            MonitorThread::init(&iface, thread_rx, thread_tx).run();
        });
    }

    fn quit(&self){
        if let Some(ipc) = self.monitor_ipc_channels.as_ref(){
            ipc.tx.send(IPCMessage::EndCommunication);
        }
        if let Some(ipc) = self.attack_ipc_channels.as_ref(){
            ipc.tx.send(IPCMessage::EndCommunication);
        }
    }

    fn randomize_theme(&mut self){
        let mut rng = rand::thread_rng();
        let rand_int = rng.gen_range(1..4);
        let theme;
        match rand_int{
            1=>{theme = colorscheme::Theme::sunny();}
            2=>{theme = colorscheme::Theme::matrix();}
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




