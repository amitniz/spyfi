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
    iface_list: StatefulList<String>
}

impl Default for WelcomeScreen{
    fn default() -> Self {
        WelcomeScreen{
            iface_list: StatefulList::new(wlan::list_interfaces()),
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
        let welcome_text:Vec<Spans> = LOGO.into_iter().map(|s|Spans::from(vec![Span::raw(s)])).collect();

        let welcome_block = Paragraph::new(welcome_text)
            .block(Block::default())
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(colorscheme::White));
        let ifaces = List::new(self.iface_list.items.iter().map(|i|{ListItem::new(format!(" 🍕 {} ",i))}).collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Interfaces ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colorscheme::White).bg(colorscheme::Green))
            )
            .style(Style::default().bg(colorscheme::LightGreen).fg(colorscheme::Purple))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(colorscheme::White).bg(colorscheme::Purple));
        f.render_widget(welcome_block, chunks[0]);
        f.render_stateful_widget(ifaces, chunks[1], &mut self.iface_list.state);
    }
   

    fn handle_input(&mut self,key:KeyEvent) -> bool{
        match key.code {
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => self.iface_list.unselect(),
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => self.iface_list.next(),
                            KeyCode::Up |KeyCode::Char('k') | KeyCode::Char('K') => self.iface_list.previous(),
                            _ => return false
        }
        true
    }
}


