use tui::style::Color;

#[derive(Clone)]
pub struct Theme{
    pub name: String,
    pub logo: Color,
    pub accent: Color,
    pub border_bg: Color,
    pub border_fg: Color,
    pub bg: Color,
    pub text: Color,
    pub bright_text: Color,
    pub highlight: Color,
    pub error: Color,
}

impl Default for Theme{
    fn default() -> Self {
        Theme::default()
    }
}

impl Theme{

    pub fn default() -> Self{
        Theme{
            name: "default".to_owned(),
            logo: Color::White,
            border_bg: Color::White,
            accent: Color::DarkGray,
            border_fg: Color::DarkGray,
            bg: Color::Gray,
            text: Color::White,
            bright_text: Color::Black,
            highlight: Color::DarkGray,
            error: Color::Red,
        }
    }
    pub fn desert() -> Self{
        Theme{
            name: "desert".to_owned(),
            logo: Color::Rgb(80, 162, 167),
            border_bg: Color::Rgb(80, 162, 167),
            accent: Color::Rgb(155, 41, 21),
            border_fg: Color::Rgb(155, 41, 21),
            bg: Color::Rgb(233, 180, 76),
            text: Color::Rgb(228, 214, 167),
            bright_text: Color::Rgb(228, 214, 167),
            highlight: Color::Rgb(80, 162, 167),
            error: Color::Rgb(155, 41, 21),
        }
    }
    pub fn eggplant() -> Self{
        Theme{
            name: "eggplant".to_owned(),
            logo: Color::Rgb(127,178,133),
            border_bg: Color::Rgb(127,178,133),
            accent: Color::Rgb(127,178,133),
            border_fg: Color::Rgb(127,178,133),
            bg: Color::Rgb(208,214,181),
            bright_text: Color::Rgb(210,210,210),
            text: Color::Rgb(152,114,132),
            highlight: Color::Rgb(152,114,132),
            error: Color::Rgb(238,118,116),
        }
    }
    
    pub fn forest() -> Self{
        Theme{
            name: "forest".to_owned(),
            logo: Color::Rgb(125, 205, 133),
            border_bg: Color::Rgb(125, 205, 133),
            accent: Color::Rgb(125, 205, 133),
            border_fg: Color::Rgb(125, 205, 133),
            bg: Color::Rgb(194, 225, 194),
            text: Color::Rgb(186, 235, 190),
            bright_text: Color::Rgb(119, 132, 114),
            highlight: Color::Rgb(128, 171, 130),
            error: Color::Rgb(238,118,116),
        }
    }
}

