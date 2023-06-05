use std::collections::HashMap;

use crate::GlobalConfigs;
use super::*;
use monitor::IPCMessage;
use crossterm::style::style;
use wpa::NetworkInfo;


pub struct MainScreen{
    tabs: StatefulList<String>,
    toggle_configs: bool,
    networks_info: HashMap<String, NetworkInfo>,
    theme: colorscheme::Theme,
}

impl Default for MainScreen{
    fn default() -> Self {
        MainScreen{
            tabs: StatefulList::new(vec!["Enum","Attack"].into_iter().map(|i|{i.to_owned()}).collect()),
            networks_info: HashMap::new(),
            theme: colorscheme::Theme::default(),
            toggle_configs: false,
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
            true => 75,
            false => 95,
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
   

    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => self.tabs.previous(),
                            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => self.tabs.previous(),
                            KeyCode::Char('c') |KeyCode::Char('C') => {self.toggle_configs = !self.toggle_configs;},
                            _ => return false
        }
        true
    }

    fn update(&mut self,ipc_msg:monitor::IPCMessage) {
        if let IPCMessage::NetworkInfo(netinfo) = ipc_msg{
            self.networks_info = netinfo;     
        }
    }

    fn set_theme(&mut self, theme:Theme) {
        self.theme = theme;
    }
    fn theme_name(&mut self) -> &str {
        &self.theme.name
    }
}

impl MainScreen{
    fn create_tabs_block<B>(&self,f:&mut Frame<B>,area: Rect) where B:Backend{
        
        let tab_names = self.tabs.items
            .iter()
            .map(|t|{            let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(first, Style::default().fg(Color::Yellow)),
                        Span::styled(rest, Style::default().fg(Color::Green)),
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
    
    fn create_configs_block<B>(&self,f:&mut Frame<B>, area: Rect) where B:Backend{
            let configs_block = Paragraph::new(
                vec![
                    Spans::from(format!("interface: {}",GlobalConfigs::get_instance().get_iface()))
                ]
            )
            .block(
                Block::default()
                    .title(" Configurations ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.fg).bg(self.theme.bg))
            )
            .style(Style::default().bg(self.theme.bg2).fg(self.theme.fg2));
            f.render_widget(configs_block, area);
    }

    fn draw_enum_tab<B>(&self, f: &mut Frame<B>,area: Rect) where B:Backend {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20),Constraint::Percentage(80)].as_ref())
            .split(area);
        let networks_block =List::new(self.networks_info.iter().map(|(k,_)|{ListItem::new(format!(" {} ",k))}).collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Networks ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.fg).bg(self.theme.bg))
            )
            .style(Style::default().bg(self.theme.bg2).fg(self.theme.fg2))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.fg)
                    .bg(self.theme.highlight)
            );
        let clients_block = Block::default().borders(Borders::ALL).title(" Clients ").border_style(Style::default().fg(self.theme.fg).bg(self.theme.bg));
        f.render_widget(networks_block, chunks[0]);
        f.render_widget(clients_block, chunks[1]);
    }
}

