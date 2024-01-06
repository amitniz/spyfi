use tui::style::{Color,Style,Modifier};

#[derive(Clone)]
pub struct Theme{
    pub name: String, // Name of the theme
    pub logo: Color, // Color of the logo
    pub border_bg: Color, // Color of the border
    pub highlight: Color, // Color of the border highlighted
    pub border_fg: Color, // Color of the border text
    pub bg: Color, // Color of the background
    pub text: Color, // Color of the text
    pub highlight_text: Color, // Color of the text highlighted
    pub popup_bg: Color, // Color of that popup background
}

impl Default for Theme{
    fn default() -> Self {
        Theme::default()
    }
}

impl Theme{
   
    pub fn logo_style(&self) -> Style {
        Style::default().fg(self.logo)
    }

    pub fn highlight_style(&self) -> Style{
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(self.highlight_text)
            .bg(self.highlight)
    }
    pub fn style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.text)
    }

    pub fn popup_style(&self) -> Style{
        Style::default().bg(self.popup_bg)
            .fg(self.highlight_text).add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self, highlight:bool) -> Style{
        let bg = match highlight{
            true => self.highlight,
            false => self.border_bg,
        };
        let fg = match highlight{
            true => self.highlight_text,
            false => self.border_fg,
        };
        Style::default().fg(fg).bg(bg)
    }
    pub fn default() -> Self{
        Theme{
            name: "default".to_owned(),
            logo: Color::Rgb(0x9d,0x02,0x08),
            border_bg: Color::Black,
            border_fg: Color::White,
            bg: Color::Black,
            text: Color::LightRed,
            highlight_text: Color::White,
            highlight: Color::Rgb(0x9d,0x02,0x08),
            popup_bg: Color::LightRed,
        }
    }


    pub fn matrix() -> Self{
        Theme{
            name: "matrix".to_owned(),
            logo: Color::Rgb(127,178,133),
            border_bg: Color::Rgb(32,72,41),
            border_fg: Color::Rgb(34,250,85),
            bg: Color::Rgb(0x5,0x38,00),
            highlight_text: Color::Rgb(0x8,0x58,1),
            text: Color::Rgb(34,180,85),
            highlight: Color::Rgb(0x10,0xa9,2),
            popup_bg: Color::Rgb(32,72,41),
        }
    }

    pub fn pastel() -> Self{
        Theme{
            name: "pastel".to_owned(),
            logo: Color::Rgb(0xa3,0xbc,0xa5),
            border_bg: Color::Rgb(0xb6,0x8e,0x5d),
            border_fg: Color::Rgb(0xa3,0xbc,0xa5),
            bg: Color::Rgb(0x6b,0x79,0x75),
            highlight_text: Color::Rgb(0x43,0x04b,0x49),
            text: Color::Rgb(0x4c,0x63,0x5c),
            highlight: Color::Rgb(0xa3,0xbc,0xa5),
            popup_bg: Color::Rgb(0xbb,0x62,0x5a),
        }
    }
}

