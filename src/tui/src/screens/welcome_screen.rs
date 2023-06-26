use std::collections::HashMap;

use crate::GlobalConfigs;
use crate::create_list;
use super::*;
use wlan;
use wpa::NetworkInfo;
const LOGO: [&'static str; 7] = [

    "███████╗██████╗ ██╗   ██╗███████╗██╗",
    "██╔════╝██╔══██╗╚██╗ ██╔╝██╔════╝██║",
    "███████╗██████╔╝ ╚████╔╝ █████╗  ██║",
    "╚════██║██╔═══╝   ╚██╔╝  ██╔══╝  ██║",
    "███████║██║        ██║   ██║     ██║",
    "╚══════╝╚═╝        ╚═╝   ╚═╝     ╚═╝",
    "             👻WiFi exploitation kit"
];

pub struct WelcomeScreen{
    iface_list: StatefulList<String>,
    theme: Theme,
}

impl Default for WelcomeScreen{
    fn default() -> Self {
        WelcomeScreen{
            iface_list: StatefulList::new(wlan::list_interfaces()),
            theme: GlobalConfigs::get_instance().theme
                .read()
                .unwrap()
                .clone()
            ,
        }
    }
}

impl<B:Backend> Screen<B> for WelcomeScreen{

    fn set_layout(&mut self, f: &mut Frame<B>) { 
       
        let w_size = Rect{
            //for a better resize response
            width: f.size().width.min(40),
            ..f.size()
        };
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(w_size.height/8)
        .constraints([Constraint::Percentage(40),Constraint::Percentage(30),Constraint::Percentage(30)].as_ref())
        .split(Rect {
            //calcultes the location of the center
            x: (f.size().width - w_size.width)/2,
            y: (f.size().height - w_size.height)/2,
            width: w_size.width,
            height: w_size.height,
        });

        // create the blocks
        self.create_logo_block(f, chunks[0]);
        let ifaces_block = create_list!(self," Interfaces ", self.iface_list.items);
        let footer_block = self.create_footer_block();
        
        // render the blocks
        f.render_stateful_widget(ifaces_block, chunks[1], &mut self.iface_list.state.clone());
        f.render_widget(footer_block, chunks[2]);
    }
   

    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => self.iface_list.next(),
                            KeyCode::Up |KeyCode::Char('k') | KeyCode::Char('K') => self.iface_list.previous(),
                            KeyCode::Enter => {self.select_iface(); return false},
                            _ => return false // handle the key outside
        }
        true // no need to handle the key globaly
    }

    fn set_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn update(&mut self,ipc_msg:ScreenIPC) -> Option<ScreenIPC>{
        //do nothing
        None
    }

}

impl WelcomeScreen{

    fn select_iface(&mut self){
        if let Some(selected_indx) = self.iface_list.state.selected(){
            let selected_iface = self.iface_list.items[selected_indx].clone();
            GlobalConfigs::get_instance().set_iface(&selected_iface);
            let channel = wlan::get_iface_channel(&selected_iface).unwrap();
            //switch to monitor mode
            wlan::toggle_monitor_state(&selected_iface, true);
            GlobalConfigs::get_instance().set_mode("monitor");
            GlobalConfigs::get_instance().set_channel(&format!("{}",channel));
        }
    }

    fn create_logo_block<B>(&self,f:&mut Frame<B>,area: Rect) where B:Backend {
    
        let welcome_text:Vec<Spans> = LOGO.into_iter().map(|s|Spans::from(vec![Span::raw(s)])).collect();
        let logo = Paragraph::new(welcome_text)
            .block(Block::default())
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(self.theme.logo));

        f.render_widget(logo,area);
    }


    fn create_footer_block(&self) -> Paragraph{

        let footer_text:Vec<Spans> = vec![Spans::from(vec![Span::raw("select<Enter>, random theme<p>, quit<q>")])];
        Paragraph::new(footer_text)
            .wrap(Wrap{trim:false})
            .block(Block::default())
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(self.theme.logo))
    }
}
