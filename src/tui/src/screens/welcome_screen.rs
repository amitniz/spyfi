use super::*;
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
        
        let w_size = f.size();
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(10)
        .horizontal_margin(105)
        .constraints([Constraint::Percentage(30),Constraint::Percentage(30),Constraint::Percentage(40)].as_ref())
        .split(Rect {
            x: 0,
            y: 0,
            width: w_size.width,
            height: w_size.height,
        });
        let welcome_text:Vec<Spans> = LOGO.into_iter().map(|s|Spans::from(vec![Span::raw(s)])).collect();

        let welcome_block = Paragraph::new(welcome_text).block(Block::default()).alignment(tui::layout::Alignment::Center);
        let ifaces = List::new(self.iface_list.items.iter().map(|i|{ListItem::new(format!(" 🎸 {} ",i))}).collect::<Vec<ListItem>>())
            .block(
                Block::default()
                    .title(" Interfaces ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White).bg(Color::Green))
            )
            .style(Style::default().bg(Color::White).fg(Color::DarkGray))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Gray));
        f.render_widget(welcome_block, chunks[0]);
        f.render_stateful_widget(ifaces, chunks[1], &mut self.iface_list.state);
    }
   

    fn handle_input(&mut self,key:KeyEvent){
        match key.code {
                            KeyCode::Left => self.iface_list.unselect(),
                            KeyCode::Down => self.iface_list.next(),
                            KeyCode::Up => self.iface_list.previous(),
                            _ => {}
        }
    }
}


