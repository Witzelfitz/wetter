use ratatui::style::{Color, Style};

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub active: Color,
    pub sun: Color,
    pub rain: Color,
    pub border: Color,
}
impl Theme {
    pub fn neu() -> Self {
        let farbe = |r, g, b| {
            if std::env::var_os("NO_COLOR").is_some() {
                Color::Reset
            } else {
                Color::Rgb(r, g, b)
            }
        };
        Self {
            bg: farbe(16, 21, 29),
            surface: farbe(24, 33, 44),
            text: farbe(230, 237, 243),
            muted: farbe(154, 170, 189),
            active: farbe(101, 214, 192),
            sun: farbe(242, 198, 109),
            rain: farbe(126, 184, 246),
            border: farbe(61, 76, 93),
        }
    }
    pub fn basis(self) -> Style {
        Style::default().fg(self.text).bg(self.bg)
    }
}
