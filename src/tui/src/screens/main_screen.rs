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
    networks: Cell<Networks>,
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
            networks: Cell::new(Networks::default()),
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

    fn set_layout(&mut self, f: &mut Frame<B>) { 
        
        let w_size = Rect{
            //for a better resize response
            ..f.size()
        };
        
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
            x: (f.size().width - w_size.width)/2,
            y: (f.size().height - w_size.height)/2,
            width: w_size.width,
            height: w_size.height,
        });
        

        //TODO: remove tabs code
        //create tabs block
        self.draw_main_window(f,chunks[0]);

        //configs block
        if self.toggle_configs{
            self.create_configs_block(f, chunks[1]);
        }

    }
   
    fn set_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
            KeyCode::Char(c) if self.panes.selected() == "attack"=>{
                let bssid: BSSID = self.networks.get_mut().get_selected_network().unwrap();
                let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                wordlist.push(c);
            }

            KeyCode::Backspace if self.panes.selected() == "attack" =>{
                let bssid = self.networks.get_mut().get_selected_network().unwrap();
                let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                wordlist.pop();
            }

            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') if !self.toggle_deauth_popup => {
                match self.panes.selected().as_str(){
                    "networks" =>{
                        self.networks.get_mut().select_previous_network(); //select previous networks
                    },
                    "clients" =>{
                        let bssid = self.networks.get_mut().get_selected_network().unwrap();
                        if !self.networks.get_mut().has_clients(&bssid){
                            self.networks.get_mut().select_previous_client(&bssid);
                        }
                    }
                    _ =>{},
                }
            },
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') if !self.toggle_deauth_popup => {
                match self.panes.selected().as_str(){
                    "networks" =>{
                        self.networks.get_mut().select_next_network(); //select next networks
                    },
                    "clients" => {
                        let bssid = self.networks.get_mut().get_selected_network().unwrap();
                        if !self.networks.get_mut().has_clients(&bssid){
                            self.networks.get_mut().select_next_client(&bssid);
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

            KeyCode::Char('d') | KeyCode::Char('D') =>{
                self.toggle_deauth_popup = true;
            }

            KeyCode::Enter if self.toggle_deauth_popup => {
                //send deauth command to monitor thread
                let bssid = self.networks.get_mut().get_selected_network().unwrap();
                let station_channel = self.networks.get_mut().get_selected_network_info()
                                                                .unwrap().channel.unwrap();
                let deauth_attack = DeauthAttack{
                    bssid,
                    client: None,
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
            KeyCode::Char('1') |  KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') |
            KeyCode::Char('5') | KeyCode::Char('6') | KeyCode::Char('7') | KeyCode::Char('8') if !self.toggle_deauth_popup => {

                if self.panes.selected().as_str() == "configs"{
                    let channel = if let KeyCode::Char(i) = key.code{
                        self.out_msg = Some(IPCMessage::IOCommand(IOCommand::ChangeChannel(i.to_digit(10).unwrap() as u8)));    
                        i
                    }else{'0'};
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
                    let bssid = self.networks.get_mut().get_selected_network().unwrap();
                    if !self.attacks.get_mut().contains_key(&bssid){
                        self.panes.next();
                    }
                }
                return true;
            },
            
            _ => return false
        }
        true
    }

    fn update(&mut self,ipc_msg: ScreenIPC) -> Option<ScreenIPC>{
        match ipc_msg{
            IPCMessage::Message(netinfo) => {
            self.networks.get_mut().update_networks(netinfo);
            },
            IPCMessage::Attack(AttackMsg::Progress(progress)) =>{
                todo!();
            },
            IPCMessage::Attack(AttackMsg::Password(password)) =>{
                todo!();
            },
            IPCMessage::Attack(AttackMsg::Exhausted) =>{
                todo!();
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
        let bg = match self.panes.selected().as_str(){
            "configs" => {self.theme.highlight},
            _ => {self.theme.border_bg},
        }; 
        //*
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
                .border_style(Style::default().fg(self.theme.border_fg).bg(bg))
        )
        .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
        f.render_widget(configs_block, area);
    }

    //draw deauth popup 
    fn draw_deauth_popup<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_fg).bg(self.theme.border_bg))
            .style(Style::default().bg(Color::Gray).fg(self.theme.text)); //TODO: fix theme


        let centered_area = Rect{
            x: area.x+2,
            y: area.y + area.height/3,
            width: area.width-4,
            height: area.height/4,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(85),
                ]
            )
            .split(centered_area);

        let device = "all devices".to_owned(); //TODO: check selected client

        let ssid: String = self.networks.get_mut().get_selected_network_info().unwrap().ssid.clone();
        let channel = self.networks.get_mut().get_selected_network_info().unwrap().channel.unwrap();

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
                Style::default()
                    .bg(Color::Gray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
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
        if !self.networks.get_mut().is_empty(){
            let mut netinfo = self.networks.get_mut().get_selected_network_info().unwrap().clone();
            self.update_network_info_pane(f,chunks[1],&netinfo);
            self.update_attack_pane(f,chunks[2],&netinfo);
        }
    
        if self.toggle_deauth_popup{
            let centered_rect = Rect{
                x: area.x+area.width/4,
                y: area.y+area.height/4,
                width: area.width/2,
                height: area.height/2,
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
            .border_style(Style::default()
                .fg(self.theme.border_fg).bg(self.theme.border_bg)
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
        f.render_widget(network_info_block, area);
    }

    fn draw_attack_pane<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend{

        let attack_bg = match self.panes.selected().as_str(){
            "attack" => {self.theme.highlight},
            _ => {self.theme.bg},
        }; 


        let attack_block = Block::default().
            borders(Borders::ALL)
            .title(" Attack ")
            .border_style(Style::default()
                .fg(self.theme.border_fg).bg(attack_bg)
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));

        f.render_widget(attack_block, area);
    }
    fn draw_networks_pane<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {
    
        //highlight the border for current pane
        let networks_bg = match self.panes.selected().as_str(){
            "networks" => {self.theme.highlight},
            _ => {self.theme.bg},
        }; 

        let networks_block = List::new(self.networks.get_mut().networks_info.iter()
            .map(|(_,v)|{ListItem::new(format!(" {} ",v.ssid))})
            .collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Networks ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_fg).bg(networks_bg))
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.bright_text)
                    .bg(self.theme.highlight)
        );
        f.render_stateful_widget(networks_block, area,&mut self.networks.get_mut().get_networks_state());
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
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
            );
            f.render_widget(text, new_area);
            return;
        }   

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3),Constraint::Percentage(75)])
            .split(Rect{
                x: area.x + 1,
                y: area.y + 1,
                width:area.width -2,
                height: area.height -2,
            });

        let attacks = self.attacks.get_mut();
        let bssid = hex::encode(network_info.bssid);
        if !attacks.contains_key(&bssid){
            attacks.insert(bssid.clone(),AttackInfo::new(network_info.handshake.as_ref().unwrap().clone(),"",1));
            self.panes.add_pane("attack");
        }
        //get the corespond DictionaryAttack
        let attack = attacks.get_mut(&bssid).unwrap();
        
        let wordlist_block = Paragraph::new(
            vec![
                Spans::from(attack.wordlist.clone())
            ]
        ).block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_fg).bg(Color::Black))
            .title(format!(" Wordlist "))).style(Style::default().bg(Color::DarkGray).fg(Color::White));


        f.render_widget(wordlist_block, chunks[0]);
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

        let clients_bg = match self.panes.selected().as_str(){
            "clients" => {self.theme.highlight},
            _ => {self.theme.bg},
        }; 

        let epoch_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let stats_block = Paragraph::new(
            vec![
                Spans::from(format!(" ssid: {}", network_info.ssid.clone())),
                Spans::from(format!(" bssid: {}",encode(network_info.bssid))),
                Spans::from(format!(" channel: {}",network_info.channel.unwrap())),
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
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
           
        let clients_block = List::new(
                network_info.clients.iter().map(|&s|ListItem::new(format!(" {} ",encode(s)))).collect::<Vec<ListItem>>()
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_fg).bg(clients_bg))
                    .title(" Clients ")
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.bright_text)
                    .bg(self.theme.highlight));
        let bssid = self.networks.get_mut().get_selected_network().unwrap();
        //render widgets
        f.render_widget(stats_block, chunks[0]);
        f.render_stateful_widget(clients_block, chunks[1],self.networks.get_mut().get_clients_state(&bssid));
        
    }
}

//TODO: convert networks to selection. Should store every selection,
//current network, client, wordlist/thread
//Note only clients must be stateful
#[derive(Debug,Clone,Default)]
struct Networks{
    pub networks_info: HashMap<BSSID,NetworkInfo>,
    networks_state: StatefulList<BSSID>,
    clients_state: HashMap<BSSID, StatefulList<String>>,
}

impl Networks{
    // returns reference to the selected NetworkInfo
    pub fn get_selected_network_info(&self) -> Option<&NetworkInfo>{
        let bssid: &BSSID = self.networks_state.selected()?;
        self.networks_info.get(bssid)
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

    pub fn select_previous_client(&mut self,bssid: &str){
        self.clients_state.get_mut(bssid).unwrap().next(); 
    }
    pub fn update_networks(&mut self, networks: HashMap<BSSID, NetworkInfo>){
        self.networks_info = networks;
        self.networks_state.items = self.networks_info.keys().cloned().collect();
        //create StatefulList of clients for each network
        self.clients_state = self.networks_info.iter().map(|(k,v)|{
            (k.clone(),
                StatefulList::new(
                    v.clients.iter().map(hex::encode).collect()
                )
            )
        }
        ).collect();
        
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

}
