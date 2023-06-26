use std::{collections::HashMap};

use crate::GlobalConfigs;
use super::*;
use wpa::NetworkInfo;
use hex::encode;
use aux::IOCommand;

pub struct MainScreen{
    tabs: StatefulList<String>,
    toggle_configs: bool,
    panes: Panes,
    networks_info: HashMap<String, NetworkInfo>,
    networks: StatefulList<String>,
    theme: colorscheme::Theme,
    out_msg: Option<ScreenIPC>,
}

impl Default for MainScreen{
    fn default() -> Self {
        MainScreen{
            tabs: StatefulList::new(vec!["Enum","Attack"].into_iter().map(|i|{i.to_owned()}).collect()),
            networks_info: HashMap::new(),
            networks: StatefulList::default(),
            theme: GlobalConfigs::get_instance().theme
                .read()
                .unwrap()
                .clone(),
            toggle_configs: false,
            panes: Panes::default(),
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
                75
            },
            false => {
                if self.panes.remove_pane("configs"){
                    //choose the first pane
                    self.panes.selected = 0;
                }
                95
            },
        };

        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(5),Constraint::Percentage(tab_view_percentage),Constraint::Percentage(20)].as_ref())
        .split(Rect {
            //calcultes the location of the center
            x: (f.size().width - w_size.width)/2,
            y: (f.size().height - w_size.height)/2,
            width: w_size.width,
            height: w_size.height,
        });
        

        //TODO: remove tabs code
        //create tabs block
        self.create_tabs_block(f,chunks[0]); 

        // render the selected tab
        match self.tabs.state.selected().unwrap_or(0){
            0 => self.draw_enum_tab(f,chunks[1]),
            1 => {},
            _ => panic!("rendered none existed tab")
        }

        //configs block
        if self.toggle_configs{
            self.create_configs_block(f, chunks[2]);
        }

    }
   
    fn set_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => self.tabs.previous(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => self.tabs.previous(),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
              match self.panes.selected().as_str(){
                "networks" =>{
                    self.networks.previous(); //select previous networks
                },
                _ =>{},
            }},
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
              match self.panes.selected().as_str(){
                "networks" =>{
                    self.networks.next(); //select next networks
                },
                _ =>{},
            }},
            //open configs panel
            KeyCode::Char('c') |KeyCode::Char('C') => {self.toggle_configs = !self.toggle_configs;
            },
            

            //channel number
            KeyCode::Char('1') |  KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') |
            KeyCode::Char('5') | KeyCode::Char('6') | KeyCode::Char('7') | KeyCode::Char('8') => {
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
                if self.panes.selected().as_str() == "configs"{
                    self.out_msg = Some(IPCMessage::IOCommand(IOCommand::Sweep));
                    GlobalConfigs::get_instance().set_channel("sweep");
                }
            }
            KeyCode::Tab => {self.panes.next()},
            
            _ => return false
        }
        true
    }

    fn update(&mut self,ipc_msg: ScreenIPC) -> Option<ScreenIPC>{
        if let IPCMessage::Message(netinfo) = ipc_msg{
            self.networks_info = netinfo;     
            let current_state = self.networks.state.clone();
            self.networks = StatefulList::new(self.networks_info.iter().map(|(k,_)|{k.clone()}).collect::<Vec<String>>());
            self.networks.state = current_state;
        }
        //send current msg and erase it
        let out_msg = self.out_msg.clone();
        self.out_msg = None;
        out_msg
    }

}

impl MainScreen{
    fn create_tabs_block<B>(&self,f:&mut Frame<B>,area: Rect) where B:Backend{
        
        let tab_names = self.tabs.items
            .iter()
            .map(|t|{            let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(first, Style::default()),
                        Span::styled(rest, Style::default()),
                    ])}
                )
            .collect();

        let tabs = Tabs::new(tab_names)
            .block(Block::default().borders(Borders::NONE))
            .select(self.tabs.state.selected().unwrap_or(0))
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );

        f.render_widget(tabs, area);

    }
    
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
        //add panes
    }

    fn draw_enum_tab<B>(&mut self, f: &mut Frame<B>,area: Rect) where B:Backend {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20),Constraint::Percentage(80)].as_ref())
            .split(area);

        let networks_bg = match self.panes.selected().as_str(){
            "networks" => {self.theme.highlight},
            _ => {self.theme.bg},
        }; 

        let info_bg = match self.panes.selected().as_str(){
            "info" => {self.theme.highlight},
            _ => {self.theme.bg},
        }; 
        
        let networks_block = List::new(self.networks.items.iter().map(|i|{ListItem::new(format!(" {} ",i))}).collect::<Vec<ListItem>>())

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
        let network_info_block = Block::default().
            borders(Borders::ALL)
            .title(" Network Info ")
            .border_style(Style::default()
                .fg(self.theme.border_fg).bg(info_bg));
        f.render_stateful_widget(networks_block, chunks[0],&mut self.networks.state);
        f.render_widget(network_info_block, chunks[1]);
        //draw info pane
        if !self.networks.items.is_empty(){
            let current_network = self.networks.items[self.networks.state.selected().unwrap_or(0)].clone();
            let netinfo = self.networks_info.get(&current_network).unwrap().clone();
            self.draw_info_pane(f,chunks[1],&netinfo);
        }

        //add new panes
        self.panes.add_pane("networks");
        self.panes.add_pane("info");
    }

    fn draw_info_pane<B>(&mut self, f: &mut Frame<B>,area: Rect,network_info: &NetworkInfo) where B:Backend {

        let area = Rect{
            x:area.x+1,
            y:area.y+1,
            width:area.width-2,
            height:area.height-2,
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30),Constraint::Percentage(70)])
            .split(area);

        let stats_block = Paragraph::new(
            vec![
                Spans::from(format!(" bssid: {}",encode(network_info.bssid))),
                Spans::from(format!(" channel: {}",network_info.channel.unwrap())),
                Spans::from(format!(" signal: {}",aux::signal_icon(network_info.signal_strength.unwrap()))),
                Spans::from(format!(" handshake: no")),
            ]
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_fg).bg(self.theme.border_bg))
                    .title(format!(" {} ", network_info.ssid.clone()))
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
           
        let clients_block = List::new(
                network_info.clients.iter().map(|&s|ListItem::new(format!(" {} ",encode(s)))).collect::<Vec<ListItem>>()
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_fg).bg(self.theme.border_bg))
                    .title(" Clients ")
            )
            .style(Style::default().bg(self.theme.bg).fg(self.theme.text));
        f.render_widget(stats_block, chunks[0]);
        f.render_widget(clients_block, chunks[1]);
    }
}

