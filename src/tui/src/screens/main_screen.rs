/*
*   The file contains the code for the main screen of the TUI
*/


use std::{collections::HashMap,cell::Cell};

use crate::GlobalConfigs;
use super::*;
use wpa::{NetworkInfo,AttackInfo};
use hex::encode;
use std::time::{SystemTime, UNIX_EPOCH};
use threads::ipc::{IOCommand,AttackMsg,DeauthAttack};

type AttacksDict = Cell<HashMap<String,AttackInfo>>;
type BSSID = String;

pub struct MainScreen{
    // show config pane
    toggle_configs: bool,
    // show deauth popup
    toggle_deauth_popup: bool,
    // screen panes
    panes: Panes,
    // captured networks info and states
    screen_selections: Cell<ScreenSelections>,
    // current attacks
    attacks: AttacksDict,
    // current theme
    theme: colorscheme::Theme,
    // msg to sent to monitor thread
    out_msg: Option<ScreenIPC>,
}

impl Default for MainScreen{

    fn default() -> Self {
        MainScreen{
            screen_selections: Cell::new(ScreenSelections::default()),
            theme: GlobalConfigs::get_instance().theme
                .read()
                .unwrap()
                .clone(),
            toggle_configs: false,
            toggle_deauth_popup: false,
            panes: Panes::default(),
            attacks: Cell::new(HashMap::new()),
            out_msg: None,
        }
    }
}


impl<B:Backend> Screen<B> for MainScreen{
    

    //determine how to draw each frame
    fn set_layout(&mut self, f: &mut Frame<B>) { 
        
        //update the tab_view size according to the appearance of the configs block
        let tab_view_percentage = match self.toggle_configs{
            true =>{ 
                if self.panes.add_pane("configs"){
                    //select the configs pane only when poped
                    self.panes.selected = self.panes.panes.len() -1;
                }
                80
            },
            false => {
                if self.panes.remove_pane("configs"){
                    //choose the first pane
                    self.panes.selected = 0;
                }
                100
            },
        };

        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(tab_view_percentage),Constraint::Percentage(20)].as_ref())
        .split(Rect {
            //calcultes the location of the center
            x: (f.size().width - f.size().width)/2,
            y: (f.size().height - f.size().height)/2,
            width: f.size().width,
            height: f.size().height,
        });
        
        //draws the main_window
        self.draw_main_window(f,chunks[0]);

        //configs block
        if self.toggle_configs{
            self.create_configs_block(f, chunks[1]);
        }
    }
  
    //sets the current theme 
    fn set_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    // handles keyboard events
    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
            KeyCode::Enter if self.panes.selected() == "attack" && !self.screen_selections.get_mut().is_currently_attacking() =>{
                let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                let mut attack: &mut AttackInfo = self.attacks.get_mut().get_mut(&bssid).unwrap();
                if attack.wordlist.len() == 0 { return true} //make sure wordlist string is not empty
                attack.attack(); //set the attack status to active
                self.screen_selections.get_mut().attack(&bssid);
                self.out_msg = Some(IPCMessage::Attack(AttackMsg::DictionaryAttack(attack.clone())));
            },


            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc if self.screen_selections.get_mut().is_currently_attacking() =>{
                let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                let mut attack: &mut AttackInfo = self.attacks.get_mut().get_mut(&bssid).unwrap();
                attack.abort(); //set the attack status to inactive
                self.screen_selections.get_mut().abort_current_attack();
                self.out_msg = Some(IPCMessage::Attack(AttackMsg::Abort));
            },

            KeyCode::Right | KeyCode::Left if self.panes.selected() =="attack" =>{
                let bssid: BSSID = self.screen_selections.get_mut().get_selected_network().unwrap();
                let attack_info = self.attacks.get_mut().get_mut(&bssid).unwrap();
                attack_info.change_selection();
            },

            KeyCode::Char(c) if self.panes.selected() == "attack" && !self.screen_selections.get_mut().is_currently_attacking()=>{
                let bssid: BSSID = self.screen_selections.get_mut().get_selected_network().unwrap();
                let attack_info = self.attacks.get_mut().get_mut(&bssid).unwrap();
                if attack_info.get_input_selection() == "wordlist"{
                    let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                    wordlist.push(c);
                }

                else if c.is_digit(10) && attack_info.get_input_selection() == "threads" && !self.screen_selections.get_mut().is_currently_attacking(){
                    //add digit to current num of threads
                    let mut num_of_threads: usize = attack_info.num_of_threads as usize;
                    num_of_threads = num_of_threads*10 + c.to_digit(10).unwrap() as usize;
                    attack_info.num_of_threads = match num_of_threads < 250{
                        true => num_of_threads as u8,
                        false => 250,
                    } //prevent overflow
                    
                }
            }


            KeyCode::Backspace if self.panes.selected() == "attack" && !self.screen_selections.get_mut().is_currently_attacking()=>{
                let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                let attack_info = self.attacks.get_mut().get_mut(&bssid).unwrap();
                if attack_info.get_input_selection() == "wordlist"{
                    let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                    wordlist.pop();
                }
                
                else if attack_info.get_input_selection() == "threads"{
                    attack_info.num_of_threads = match attack_info.num_of_threads/10{
                        0 => 0,
                        n => n,
                    };
                }
            }

            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') if !self.toggle_deauth_popup => {
                match self.panes.selected().as_str(){
                    "networks" =>{
                        self.screen_selections.get_mut().select_previous_network(); //select previous networks
                    },
                    "clients" =>{
                        let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                        if !self.screen_selections.get_mut().has_clients(&bssid){
                            self.screen_selections.get_mut().select_previous_client(&bssid);
                        }
                    }
                    _ =>{},
                }
            },
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') if !self.toggle_deauth_popup => {
                match self.panes.selected().as_str(){
                    "networks" =>{
                        self.screen_selections.get_mut().select_next_network(); //select next networks
                    },
                    "clients" => {
                        let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                        if !self.screen_selections.get_mut().has_clients(&bssid){
                            self.screen_selections.get_mut().select_next_client(&bssid);
                        }
                    }
                _ =>{},
                }
            },
            //open configs panel
            KeyCode::Char('c') |KeyCode::Char('C') if !self.toggle_deauth_popup => {
                self.toggle_configs = !self.toggle_configs;
            },

            //open deauth popup
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Enter if !self.toggle_deauth_popup && self.panes.selected() != "attack" =>{
                //make sure that there are discovered networks
                if !self.screen_selections.get_mut().networks_info.is_empty() {
                    self.toggle_deauth_popup = true;
                }
            }

            KeyCode::Enter if self.toggle_deauth_popup => {
                //send deauth command to monitor thread
                let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                let station_channel = self.screen_selections.get_mut().get_selected_network_info()
                                                                .unwrap().channel.unwrap();


                let client: Option<String> = match self.panes.selected().as_str(){
                    "clients" => self.screen_selections.get_mut().get_selected_client(),
                    _ => None
                };

                let deauth_attack = DeauthAttack{
                    bssid,
                    client: client,
                    station_channel,
                };
                self.out_msg = Some(IPCMessage::Attack(AttackMsg::DeauthAttack(deauth_attack)));
                //toggle deauth popup
                self.toggle_deauth_popup = !self.toggle_deauth_popup;
            } 
            //close deauth popup
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') if self.toggle_deauth_popup =>{
                    self.toggle_deauth_popup = false;
            },

            //channel number
            KeyCode::Char(d) if d.is_ascii_digit() && !self.toggle_deauth_popup => {
                if self.panes.selected().as_str() == "configs"{
                    let channel: u8 = match d.to_digit(10) {
                        Some(0) => 10,
                        Some(d) => d as u8,
                        None => 1,
                    };
                    self.out_msg = Some(IPCMessage::IOCommand(IOCommand::ChangeChannel(channel)));    
                    GlobalConfigs::get_instance().set_channel(&format!("{}",channel));
                }  
            },
            //toggle sweep
            KeyCode::Char('s') | KeyCode::Char('S') =>{
                if self.toggle_deauth_popup{
                    return true;
                }             
                if self.panes.selected().as_str() == "configs"{
                    self.out_msg = Some(IPCMessage::IOCommand(IOCommand::Sweep));
                    GlobalConfigs::get_instance().set_channel("sweep");
                }
            }
            KeyCode::Tab => {
                if self.toggle_deauth_popup{
                    return true;
                }             
                self.panes.next();
                //don't allow attack pane with no attack info
                if self.panes.selected() == "attack"{
                    let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
                    if !self.attacks.get_mut().contains_key(&bssid){
                        self.panes.next();
                    }
                }
            },

            _ => return false //pass the event outside to the TUI struct
        }
        true
    }

    fn update(&mut self,ipc_msg: ScreenIPC) -> Option<ScreenIPC>{
        match ipc_msg{
            IPCMessage::Message(netinfo) => {
                self.screen_selections.get_mut().update_networks(netinfo);
            },
            IPCMessage::Attack(AttackMsg::Progress(progress)) =>{
                //TODO: make without unwrap and consider to remove attack_info from netinfo. maybe
                //store is attacking only, or attack id
                //find attacking network
                if let Some(bssid) = self.screen_selections.get_mut().get_current_attack_bssid(){
                    let attack_info:&mut AttackInfo = self.attacks.get_mut().get_mut(&bssid).unwrap();
                    attack_info.update(progress.size_of_wordlist,progress.num_of_attempts,progress.passwords_attempts);
                }
            },
            IPCMessage::Attack(AttackMsg::Password(password)) =>{
                let bssid = self.screen_selections.get_mut().get_current_attack_bssid().unwrap();
                let attack_info:&mut AttackInfo = self.attacks.get_mut().get_mut(&bssid).unwrap();
                attack_info.set_password(&password);
                self.out_msg = Some(IPCMessage::Attack(AttackMsg::Abort)); //kill the AttackThread
            },
            IPCMessage::Attack(AttackMsg::Exhausted) =>{
                let bssid = self.screen_selections.get_mut().get_current_attack_bssid().unwrap();
                let attack_info:&mut AttackInfo = self.attacks.get_mut().get_mut(&bssid).unwrap();
                attack_info.exhausted();
                self.out_msg = Some(IPCMessage::Attack(AttackMsg::Abort)); //kill the AttackThread
            },
            IPCMessage::Attack(AttackMsg::Error) =>{
                todo!();
            }
            _ =>{},
        }
        //send current msg and erase it
        let out_msg = self.out_msg.clone();
        self.out_msg = None;
        out_msg
    }

}

impl MainScreen{

    fn create_configs_block<B>(&mut self,f:&mut Frame<B>, area: Rect) where B:Backend{
        let configs_block = Paragraph::new(
            vec![
                Spans::from(format!(" interface: {}",GlobalConfigs::get_instance().get_iface())),
                Spans::from(format!(" mode: {}",GlobalConfigs::get_instance().get_mode())),
                Spans::from(format!(" channel: {}",GlobalConfigs::get_instance().get_channel())),
            ]
        )
        .block(
            Block::default()
                .title(" Configurations ")
                .borders(Borders::ALL)
                .border_style(self.theme.border_style(self.panes.selected()=="configs"))
        )
        .style(self.theme.style());
        f.render_widget(configs_block, area);
    }

    //draw deauth popup 
    fn draw_deauth_popup<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(false))
            .style(self.theme.popup_style()); 


        let centered_area = Rect{
            x: area.x+2,
            y: area.y + area.height/3,
            width: area.width-4,
            height: area.height/2,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),
                Constraint::Percentage(100),
                ]
            )
            .split(centered_area);

        let device = match self.panes.selected().as_str(){
            "clients" => self.screen_selections.get_mut().get_selected_client().unwrap_or("all devices".to_owned()),
            _=> "all devices".to_owned()
        }; //TODO: check selected client

        let ssid: String = self.screen_selections.get_mut().get_selected_network_info().unwrap().ssid.clone();
        let iface = GlobalConfigs::get_instance().get_iface();
        let channel = wlan::get_iface_channel(&iface).unwrap();

        let text = Paragraph::new(
            vec![
                Spans::from(format!("Are you sure you want disconnect {}",
                    device,
                )),
                Spans::from(format!("from the network {} at channel {}?",
                    ssid,
                    channel
                )),
                Spans::from(format!("")),
                Spans::from(format!("<ENTER> Ok  <ESC> Cancel ")),
            ]
        ).alignment(Alignment::Center)
        .style(
                self.theme.popup_style()
        );

        f.render_widget(Clear, area);
        f.render_widget(popup_block, area);
        f.render_widget(Clear, chunks[1]);
        f.render_widget(text, chunks[1]);

    }

    // draws the main pane graphics
    fn draw_main_window<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20),Constraint::Percentage(20),Constraint::Percentage(60)].as_ref())
            .split(area);

        //render blocks
        self.draw_networks_pane(f, chunks[0]);
        self.draw_network_info_pane(f,chunks[1]);
        self.draw_attack_pane(f,chunks[2]);
        
        //update networks info pane and attack pane
        if !self.screen_selections.get_mut().is_empty(){
            let mut netinfo = self.screen_selections.get_mut().get_selected_network_info().unwrap().clone();
            self.update_network_info_pane(f,chunks[1],&netinfo);
            self.update_attack_pane(f,chunks[2],&netinfo);
        }
    
        if self.toggle_deauth_popup{
            let centered_rect = Rect{
                x: area.x+area.width/3,
                y: area.y+area.height/3,
                width: area.width/3,
                height: area.height/3,
            };
            self.draw_deauth_popup(f, centered_rect)
        }

        //add new panes
        self.panes.add_pane("networks");
        self.panes.add_pane("clients");

    }

    fn draw_network_info_pane<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend{
        let network_info_block = Block::default().
            borders(Borders::ALL)
            .title(" Network Info ")
            .border_style(self.theme.border_style(false))
            .style(self.theme.style());
        f.render_widget(network_info_block, area);
    }

    fn draw_attack_pane<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend{

        let attack_block = Block::default().
            borders(Borders::ALL)
            .title(" Attack ")
            .border_style(self.theme.border_style(self.panes.selected() == "attack"))
            .style(self.theme.style());

        f.render_widget(attack_block, area);
    }

    fn draw_networks_pane<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {
    
        let networks_block = List::new(self.screen_selections.get_mut().networks_info.iter()
            .map(|(_,v)|{ListItem::new(format!(" {} ",v.ssid))})
            .collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Networks ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(self.panes.selected() == "networks"))
            )
            .style(self.theme.style())
            .highlight_style( self.theme.highlight_style());

        f.render_stateful_widget(networks_block, area,&mut self.screen_selections.get_mut().get_networks_state());
    }

    fn update_attack_pane<B>(&mut self, f:&mut Frame<B>, area: Rect, network_info: &NetworkInfo) where B:Backend{
        
        // show message if no handshake found.
        if network_info.handshake.is_none(){
            let new_area = Rect{
                x: area.x,
                y: area.y + area.height/2,
                width: area.width,
                height: area.height/2,
            };
            let text = Paragraph::new(
                vec![
                    Spans::from("No Handshake captured.")
                ]
            ).alignment(Alignment::Center)
            .style(
                    Style::default()
                        .fg(self.theme.text)
                        .add_modifier(Modifier::BOLD)
            );
            f.render_widget(text, new_area);
            return;
        }   

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3),Constraint::Percentage(100)])
            .split(Rect{
                x: area.x + 1,
                y: area.y + 1,
                width:area.width -2,
                height: area.height -2,
            });
        
        //TODO move out of here to update
        let attacks = self.attacks.get_mut();
        let bssid = hex::encode(network_info.bssid);
        if !attacks.contains_key(&bssid){
            let attack_info = AttackInfo::new(network_info.handshake.as_ref().unwrap().clone(),"",1);
            attacks.insert(bssid.clone(),attack_info.clone());
            self.panes.add_pane("attack");
        }
        //get the corespond DictionaryAttack
        let attack = attacks.get_mut(&bssid).unwrap();
       
        // draw attack info subpane 
        draw_attack_info(f, chunks[0], attack, &self.theme);

        if attack.is_attacking(){
            draw_attack_status(f,chunks[1], attack,&self.theme);
        }

    }

    
    

    fn update_network_info_pane<B>(&mut self, f: &mut Frame<B>,area: Rect,network_info: &NetworkInfo) where B:Backend {

        let area = Rect{
            x:area.x,
            y:area.y,
            width:area.width,
            height:area.height,
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(25),Constraint::Percentage(75)])
            .split(area);

        let epoch_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        //get the channel of the selected client if there is one
        //HERE
        let channel:u8 = match self.screen_selections.get_mut().get_selected_client(){
            Some(client_mac) =>{
                //extract client channel
                network_info.clients.iter().find(|c|c.mac ==client_mac).unwrap().channel
            },
            None => network_info.channel.unwrap(),
        };
        let stats_block = Paragraph::new(
            vec![
                Spans::from(format!(" ssid: {}", network_info.ssid.clone())),
                Spans::from(format!(" bssid: {}",encode(network_info.bssid))),
                Spans::from(format!(" channel: {}",channel)),
                Spans::from(format!(" signal: {}",aux::signal_icon(network_info.signal_strength.unwrap()))),
                Spans::from(format!(" protocol: {}",network_info.protocol)),
                Spans::from(format!(" handshake: {}",match network_info.handshake.is_some(){
                    true =>"✅",
                    false =>"❎",
                })),
                Spans::from(format!(" last appearance: {} sec",epoch_now-network_info.last_appearance)),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_fg).bg(self.theme.border_bg))
                    .title(format!(" Network Info "))
                    
            )
            .style(self.theme.style());
           
        let clients_block = List::new(
                network_info.clients.iter().map(|s|ListItem::new(format!(" {} ",s.mac))).collect::<Vec<ListItem>>()
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(self.panes.selected() == "clients"))
                    .title(" Clients ")
            )
            .style(self.theme.style())
            .highlight_style(self.theme.highlight_style());
        let bssid = self.screen_selections.get_mut().get_selected_network().unwrap();
        //render widgets
        f.render_widget(stats_block, chunks[0]);
        f.render_stateful_widget(clients_block, chunks[1],self.screen_selections.get_mut().get_clients_state(&bssid));
        
    }
}



//TODO: convert networks to selection. Should store every selection,
//current network, client, wordlist/thread
//Note only clients must be stateful
#[derive(Debug,Clone,Default)]
struct ScreenSelections{
    pub networks_info: HashMap<BSSID,NetworkInfo>,
    networks_state: StatefulList<BSSID>,
    clients_state: HashMap<BSSID, StatefulList<String>>,
    attacked_network: Option<String>,
}

impl ScreenSelections{
   
    pub fn abort_current_attack(&mut self){
        self.attacked_network = None;
    }
    pub fn is_currently_attacking(&self) -> bool {
        match self.attacked_network{
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_selected_client(&self) -> Option<String>{
        let selected_network = self.get_selected_network()?;
        let clients = &self.clients_state[&selected_network];
        clients.selected().cloned()
    }


    pub fn get_selected_client_channel(&self) ->Option<u8>{
        let selected_network = self.get_selected_network().unwrap();
        let clients = &self.clients_state[&selected_network];
        let client_mac = clients.selected()?; 
        Some(self.networks_info[&selected_network].clients.iter().find(|i|&i.mac == client_mac).unwrap().channel)
    }

    // returns reference to the selected NetworkInfo
    pub fn get_selected_network_info(&mut self) -> Option<&mut NetworkInfo>{
        let bssid: &BSSID = self.networks_state.selected()?;
        self.networks_info.get_mut(bssid)
    }

    pub fn get_network(&mut self,bssid: &str) -> Option<&mut NetworkInfo>{
        self.networks_info.get_mut(bssid)
    }  

    pub fn is_empty(&self) -> bool{
        self.networks_info.is_empty()
    }
    
    pub fn get_selected_network(&self) -> Option<BSSID>{
        Some(self.networks_state.selected()?.to_owned())
    }

    pub fn select_next_network(&mut self){
        self.networks_state.next();
    }

    pub fn select_previous_network(&mut self){
        self.networks_state.previous();
    }
    
    pub fn select_next_client(&mut self,bssid: &str){
        self.clients_state.get_mut(bssid).unwrap().next(); 
    }

    pub fn get_current_attack_bssid(&mut self) -> Option<String>{
        self.attacked_network.clone()
    }

    pub fn select_previous_client(&mut self,bssid: &str){
        self.clients_state.get_mut(bssid).unwrap().next(); 
    }
    /// update_networks
    /// ### Description:
    /// replace the previous captured networks state with a recent, updated state 
    /// provided by the Monitor thread
    pub fn update_networks(&mut self, networks: HashMap<BSSID, NetworkInfo>){
        self.networks_info = networks; //replace the old net_info with the new one

        //store the current selected network if there is one
        let selected_network = self.get_selected_network();
        //recreate StatefullList of networks
        self.networks_state.items = self.networks_info.keys().cloned().collect();
        //restore selected
        if let Some(bssid) = &selected_network{
            let idx = self.networks_state.items.iter().position(|b|b == bssid);
            self.networks_state.state.select(idx);
        }
    
        let selected_client = self.get_selected_client();
        
        //recreate StatefulList of clients for each network
        self.clients_state = self.networks_info.iter().map(|(k,v)|{
            (k.clone(),
                StatefulList::new(
                    v.clients.iter().map(|c|c.mac.clone()).collect()
                )
            )
        }
        ).collect();
       
        //restore selected client
        if let Some(bssid) = selected_client{
            let selected_network = selected_network.unwrap();
            let idx = self.clients_state.get(&selected_network).unwrap().items.iter().position(|b|b==&bssid);
            self.get_clients_state(&selected_network).select(idx);
        }
        // set a network selection if there is none
        if self.networks_state.selected().is_none() && !self.is_empty(){
            self.networks_state.next();
        }
    }

    pub fn get_networks_state(&mut self) -> &mut ListState{
        &mut self.networks_state.state
    }

    pub fn has_clients(&self,bssid:&str) -> bool{
        self.networks_info.get(bssid).unwrap().clients.is_empty()
    }

    pub fn get_clients_state(&mut self,bssid: &str) -> &mut ListState{
        let state_list = self.clients_state.get_mut(bssid).unwrap();
        &mut state_list.state
    }
    
    pub fn attack(&mut self,bssid:&str){
        self.attacked_network =Some(bssid.to_owned());
    }

}


//------------------------------ draw functions -------------------------------
////TODO: all draw function should be like that. non structs method, no changing
//states, just drawing data to screen

// draws the attack info subpane (wordlist path, therads..)
fn draw_attack_info<B>(f: &mut Frame<B>,area: Rect, attack_info: &AttackInfo, theme: &Theme) where B:Backend {

    let wordlist_border = match attack_info.get_input_selection(){
        "wordlist" => theme.border_fg,
        _ => Color::Gray,
    };

    let threads_border = match attack_info.get_input_selection(){
        "threads" => theme.border_fg,
        _ => Color::Gray,
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(50),Constraint::Length(12),Constraint::Min(40)])
        .split(area);

    // wordlist block
    let wordlist_block = Paragraph::new(
        vec![
            Spans::from(attack_info.wordlist.clone())
        ]
    ).block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(wordlist_border).bg(Color::Black))
        .title(format!(" Wordlist "))).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    f.render_widget(wordlist_block, chunks[0]);
    
    // wordlist block
    let wordlist_block = Paragraph::new(
        vec![
            Spans::from(format!("{}",attack_info.num_of_threads))
        ]
    ).block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(threads_border).bg(Color::Black))
        .title(format!(" Threads "))).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    f.render_widget(wordlist_block, chunks[1]);
}

// draws the progress of the current attack 
// TODO: use theme
fn draw_attack_status<B>(f: &mut Frame<B>,area: Rect, attack_info: &mut AttackInfo, theme: &Theme) where B:Backend{
        // split the area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3),Constraint::Length(2),Constraint::Length(1),Constraint::Percentage(50),Constraint::Percentage(10)])
            .split(area);
        // progress gauge
        let progress_gauge =Gauge::default() .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::ITALIC))
        .percent(attack_info.progress().min(100));
        f.render_widget(progress_gauge, chunks[0]);

        //progress stats
        let progress_msg = match attack_info.size_of_wordlist{
            0 => format!(" Loading wordlist.."),
            _ => format!( " Progress: {}/{}",attack_info.num_of_attempts,attack_info.size_of_wordlist)
        };

        //elapsed time
        let elapsed_time = format!(" Elapsed time: {}",attack_info.elapsed_time());
        let progress_block = Paragraph::new(
            vec![
                Spans::from(progress_msg),
                Spans::from(elapsed_time),
            ]
        ).block(Block::default());
        f.render_widget(progress_block, chunks[1]);

        if attack_info.previous_attempts.len() > 0{
            //current attempt block
            
            let current_attempt_msg = match attack_info.cracked(){
                Some(password) => format!(" Found: {password}"),
                None => {
                    if attack_info.is_exhausted(){
                        format!(" Password not found..")
                    }else{
                        format!(" Trying: {}",&attack_info.previous_attempts[0])
                    }
                }
            };

            let current_attempt = Paragraph::new(
                vec![
                    Spans::from(current_attempt_msg)
                ]
            ).block(Block::default());
            f.render_widget(current_attempt, chunks[2]);

            if !attack_info.is_exhausted(){

                let previous_attempts = Paragraph::new(
                        attack_info.previous_attempts[1..].iter().map(|i|Spans::from(format!("         {}",i.clone()))).collect::<Vec<Spans>>()
                ).block(Block::default()).style(Style::default().fg(Color::Rgb(0x6e, 0x4e,0x4e )));
                f.render_widget(previous_attempts, chunks[3]);
                //remove last attempt from list
                if attack_info.cracked().is_none(){
                    attack_info.previous_attempts.remove(0);
                }
            }
        }
       
}
