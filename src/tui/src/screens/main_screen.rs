use std::{collections::HashMap};

use crate::GlobalConfigs;
use super::*;
use wpa::{NetworkInfo, Handshake};
use hex::encode;
use std::time::{SystemTime, UNIX_EPOCH};
use aux::IOCommand;


#[derive(Clone,Debug)]
pub struct AttackInfo{
    pub handshake: Option<Handshake>,
    pub ssid: String,
    pub bssid: String,
    pub client: String,
}

pub struct MainScreen{
    toggle_configs: bool,
    toggle_deauth_popup: bool,
    panes: Panes,
    networks_info: HashMap<String, NetworkInfo>,
    networks: StatefulList<String>,
    theme: colorscheme::Theme,
    out_msg: Option<ScreenIPC>,
    attack_info: Option<AttackInfo>,
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
            out_msg: None,
            attack_info: None,
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
            KeyCode::Enter =>{
                if self.attack_info.is_some(){
                    if self.toggle_deauth_popup{
                        //send deauth
                        let iface = GlobalConfigs::get_instance().get_iface();
                        let bssid = self.attack_info.as_ref().unwrap().bssid.clone();
                        wpa::send_deauth(&iface, &bssid, None);
                    }
                    //toggle deauth popup
                    self.toggle_deauth_popup = !self.toggle_deauth_popup;
                }
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
                self.panes.next()
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

        let device = match self.attack_info.as_ref().unwrap().client.as_str(){
            "broadcast" => "all devices".to_owned(),
            s => s.to_owned(),
        };

        let network = self.attack_info.as_ref().unwrap().ssid.clone();


        let text = Paragraph::new(
            vec![
                Spans::from(format!("Are you sure you want disconnect {} from the network {}?",device,network)),
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
        
        //update networks info pane
        if !self.networks.items.is_empty(){
            let current_network = self.networks.items[self.networks.state.selected().unwrap_or(0)].clone();
            let netinfo = self.networks_info.get(&current_network).unwrap().clone();
            self.update_network_info_pane(f,chunks[1],&netinfo);
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
        let attack_block = Block::default().
            borders(Borders::ALL)
            .title(" Attack ")
            .border_style(Style::default()
                .fg(self.theme.border_fg).bg(self.theme.border_bg)
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

        //update attack info
        let attack_info = AttackInfo{
            ssid: network_info.ssid.clone(),
            bssid: hex::encode(network_info.bssid),
            client: "broadcast".to_owned(),
            handshake: network_info.handshake.clone(),
        };

        self.attack_info = Some(attack_info);
    }
}

