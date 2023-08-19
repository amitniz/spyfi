use std::{collections::HashMap,cell::Cell};

use crate::GlobalConfigs;
use super::*;
use wpa::{NetworkInfo,AttackInfo, DeauthAttack, DictionaryAttack};
use hex::encode;
use std::time::{SystemTime, UNIX_EPOCH};
use threads::ipc::IOCommand;

type AttacksDict = Cell<HashMap<String,DictionaryAttack>>;
type BSSID = String;


pub struct MainScreen{
    // show config pane
    toggle_configs: bool,
    // show deauth popup
    toggle_deauth_popup: bool,
    // screen panes
    panes: Panes,
    // captured networks
    networks_info: HashMap<String, NetworkInfo>,
    // captured stateful list
    networks: StatefulList<String>,
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
            networks_info: HashMap::new(),
            networks: StatefulList::default(),
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
                let bssid = hex::encode(self.networks_info.get(self.networks.selected().unwrap())
                    .as_ref().unwrap().bssid.clone());
                let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                wordlist.push(c);
            }

            KeyCode::Backspace if self.panes.selected() == "attack" =>{
                let bssid = hex::encode(self.networks_info.get(self.networks.selected().unwrap())
                    .as_ref().unwrap().bssid.clone());
                let mut wordlist = &mut self.attacks.get_mut().get_mut(&bssid).unwrap().wordlist;
                wordlist.pop();
            }

            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                if self.toggle_deauth_popup{
                    return true;
                }             
                match self.panes.selected().as_str(){
                    "networks" =>{
                    self.networks.previous(); //select previous networks
                },
                _ =>{},
            }},
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                if self.toggle_deauth_popup{
                    return true;
                }             
                match self.panes.selected().as_str(){
                    "networks" =>{
                    self.networks.next(); //select next networks
                },
                _ =>{},
            }},
            //open configs panel
            KeyCode::Char('c') |KeyCode::Char('C') => {
                if self.toggle_deauth_popup{
                    return true;
                }             
                self.toggle_configs = !self.toggle_configs;
            },
            //open deauth popup
            KeyCode::Enter | KeyCode::Char('d') | KeyCode::Char('D') =>{
                if self.toggle_deauth_popup{
                    //send deauth
                    let iface = GlobalConfigs::get_instance().get_iface();
                    //TODO: consider storing bssid as String in networkinfo
                    let bssid = hex::encode(self.networks_info.get(self.networks.selected().unwrap())
                        .as_ref().unwrap().bssid.clone());
                    let station_channel = self.networks_info.get(self.networks.selected().unwrap())
                        .as_ref().unwrap().channel.unwrap();
                    let deauth_attack = DeauthAttack{
                        bssid,
                        client: None,
                        station_channel,
                    };
                    self.out_msg = Some(IPCMessage::Attack(AttackInfo::DeauthAttack(deauth_attack)));
                }
                //toggle deauth popup
                self.toggle_deauth_popup = !self.toggle_deauth_popup;
            } 
            //close deauth popup
            KeyCode::Esc =>{
                    self.toggle_deauth_popup = false;
            },

            //channel number
            KeyCode::Char('1') |  KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') |
            KeyCode::Char('5') | KeyCode::Char('6') | KeyCode::Char('7') | KeyCode::Char('8') => {
                if self.toggle_deauth_popup{
                    return true;
                }             

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
                    let bssid = hex::encode(self.networks_info.get(self.networks.selected().unwrap())
                    .as_ref().unwrap().bssid.clone());
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
        if let IPCMessage::Message(netinfo) = ipc_msg{
            self.networks_info = netinfo;     
            let current_state = self.networks.state.clone();
            self.networks = StatefulList::new(self.networks_info.iter().map(|(k,_)|{k.clone()}).collect::<Vec<String>>());
            if self.networks_info.len() > 0 && self.networks.state.selected().is_none(){
                self.networks.next();
            }
            self.networks.state = current_state;
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
                Constraint::Percentage(45),
                Constraint::Percentage(35),
                Constraint::Percentage(20)]
            )
            .split(centered_area);

        let device = "all devices".to_owned(); //TODO: check selected client

        let network = self.networks_info.get(self.networks.selected().unwrap()).unwrap().ssid.clone();
        let channel = self.networks_info.get(self.networks.selected().unwrap())
            .unwrap().channel.unwrap();

        let text = Paragraph::new(
            vec![
                Spans::from(format!("Are you sure you want disconnect {} from the network {} at channel {}?",
                    device,
                    network,
                    channel
                )),
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
        if !self.networks.items.is_empty(){
            let current_network = self.networks.items[self.networks.state.selected().unwrap_or(0)].clone();
            let mut netinfo = self.networks_info.get(&current_network).unwrap().clone();
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

        let networks_block = List::new(self.networks_info.iter()
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
        f.render_stateful_widget(networks_block, area,&mut self.networks.state);
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
            attacks.insert(bssid.clone(),DictionaryAttack::default());
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
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
        //render widgets
        f.render_widget(stats_block, chunks[0]);
        f.render_widget(clients_block, chunks[1]);
        
    }
}

//TODO: convert networks to selection. Should store every selection,
//current network, client, wordlist/thread
//Note only clients must be stateful
#[derive(Debug,Clone,Default)]
struct Networks{
    network_info: HashMap<BSSID,NetworkInfo>,
    network_state: StatefulList<BSSID>,
    clients_state: HashMap<BSSID, StatefulList<String>>,
}

impl Networks{
    // returns reference to the selected NetworkInfo
    pub fn get_selected_network_info(&self) -> Option<&NetworkInfo>{
        let bssid: &BSSID = self.network_state.selected()?;
        self.network_info.get(bssid)
    }

    pub fn select_next_network(&mut self){
        self.network_state.next();
    }
    
    pub fn select_next_client(&mut self,bssid: &str){
        todo!();
    }

    pub fn update_networks(&mut self, networks: Vec<NetworkInfo>){todo!();}

    pub fn get_networks_state(&self) -> &ListState{
        &self.network_state.state
    }

    pub fn get_clients_state(&self,bssid: &str) -> Option<&ListState>{
        let state_list = self.clients_state.get(bssid)?;
        Some(&state_list.state)
    }

}
