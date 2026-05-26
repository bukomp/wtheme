// Registry encoding: 0 = Dark, 1 = Light (Windows convention for AppsUseLightTheme / SystemUsesLightTheme).

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Light,
    Dark,
}

impl Mode {
    pub fn dword(self) -> u32 {
        match self {
            Mode::Light => 1,
            Mode::Dark => 0,
        }
    }

    pub fn from_dword(v: u32) -> Self {
        if v == 0 { Mode::Dark } else { Mode::Light }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }

    pub fn invert(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }
}
