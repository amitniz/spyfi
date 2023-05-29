use tui::style::Color;

#[derive(Clone)]
pub struct Theme{
    pub name: String,
    pub logo: Color,
    pub bg: Color,
    pub bg2: Color,
    pub fg: Color,
    pub fg2: Color,
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
            bg: Color::DarkGray,
            bg2: Color::Gray,
            fg: Color::White,
            fg2: Color::Black,
            highlight: Color::DarkGray,
            error: Color::Red,
        }
    }
    pub fn desert() -> Self{
        Theme{
            name: "desert".to_owned(),
            logo: Color::Rgb(80, 162, 167),
            bg: Color::Rgb(155, 41, 21),
            bg2: Color::Rgb(233, 180, 76),
            fg: Color::Rgb(228, 214, 167),
            fg2: Color::Rgb(228, 214, 167),
            highlight: Color::Rgb(80, 162, 167),
            error: Color::Rgb(155, 41, 21),
        }
    }
    pub fn eggplant() -> Self{
        Theme{
            name: "eggplant".to_owned(),
            logo: Color::Rgb(127,178,133),
            bg: Color::Rgb(127,178,133),
            bg2: Color::Rgb(208,214,181),
            fg: Color::Rgb(210,210,210),
            fg2: Color::Rgb(152,114,132),
            highlight: Color::Rgb(152,114,132),
            error: Color::Rgb(238,118,116),
        }
    }

    pub fn pokemon() -> Self{
        Theme{
            name: "pokemon".to_owned(),
            logo: Color::Rgb(222, 84, 30),
            bg: Color::Rgb(73, 67, 49),
            bg2: Color::Rgb(214, 214, 177),
            fg: Color::Rgb(214, 214, 177),
            fg2: Color::Rgb(63,63,55),
            highlight: Color::Rgb(222, 84, 30),
            error: Color::Rgb(222, 84, 30),
        }
    }

    pub fn megaman() -> Self{
        Theme{
            name: "megaman".to_owned(),
            logo: Color::Rgb(191, 33, 30),
            bg: Color::Rgb(105, 161, 151),
            bg2: Color::Rgb(229, 249, 147),
            fg: Color::Rgb(249, 220, 92),
            fg2: Color::Rgb(191, 33, 30),
            highlight: Color::Rgb(191, 33, 30),
            error: Color::Rgb(191, 33, 30),
        }
    }
    pub fn jamaica() -> Self{
        Theme{
            name: "jamaica".to_owned(),
            logo: Color::Rgb(255, 207, 0),
            bg: Color::Rgb(0, 145, 110),
            bg2: Color::Rgb(224, 209, 209),
            fg: Color::Rgb(254, 239, 229),
            fg2: Color::Rgb(0, 145, 110),
            highlight: Color::Rgb(238, 97, 35),
            error: Color::Rgb(250, 0, 63),
        }
    }
    
    pub fn forest() -> Self{
        Theme{
            name: "forest".to_owned(),
            logo: Color::Rgb(125, 205, 133),
            bg: Color::Rgb(125, 205, 133),
            bg2: Color::Rgb(194, 225, 194),
            fg: Color::Rgb(186, 235, 190),
            fg2: Color::Rgb(119, 132, 114),
            highlight: Color::Rgb(128, 171, 130),
            error: Color::Rgb(238,118,116),
        }
    }
}

