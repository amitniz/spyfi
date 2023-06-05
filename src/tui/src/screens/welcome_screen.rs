use crate::GlobalConfigs;

use super::*;
use crossterm::style::style;
use wlan;

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
    theme: colorscheme::Theme,
}

impl Default for WelcomeScreen{
    fn default() -> Self {
        WelcomeScreen{
            iface_list: StatefulList::new(wlan::list_interfaces()),
            theme: colorscheme::Theme::default(),
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
        let logo_block = self.create_logo_block();
        let ifaces_block = self.create_ifaces_block();
        let footer_block = self.create_footer_block();
        
        // render the blocks
        f.render_widget(logo_block, chunks[0]);
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

    fn set_theme(&mut self, theme:Theme) {
        self.theme = theme;
    }

    fn theme_name(&mut self) -> &str {
        &self.theme.name
    }

    fn update(&mut self,ipc_msg:monitor::IPCMessage) {
        //do nothing
    }

}

impl WelcomeScreen{

    fn select_iface(&mut self){
        if let Some(selected_indx) = self.iface_list.state.selected(){
            let selected_iface = self.iface_list.items[selected_indx].clone();
            GlobalConfigs::get_instance().set_iface(&selected_iface);
        }
    }

    fn create_logo_block(&self) -> Paragraph{

        let welcome_text:Vec<Spans> = LOGO.into_iter().map(|s|Spans::from(vec![Span::raw(s)])).collect();
        Paragraph::new(welcome_text)
            .block(Block::default())
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(self.theme.logo))
    }

    fn create_ifaces_block<'f>(&'f self) -> List{

        List::new(self.iface_list.items.iter().map(|i|{ListItem::new(format!(" 🍕 {} ",i))}).collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Interfaces ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.fg).bg(self.theme.bg))
            )
            .style(Style::default().bg(self.theme.bg2).fg(self.theme.fg2))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(self.theme.fg).bg(self.theme.highlight))

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
