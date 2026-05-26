// Two-state theme value. Stored in the registry as a DWORD: 0 = Dark, 1 = Light.

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Light,
    Dark,
}

impl Mode {
    // Encode for the registry: matches Windows' AppsUseLightTheme / SystemUsesLightTheme convention.
    pub fn dword(self) -> u32 {
        match self {
            Mode::Light => 1,
            Mode::Dark => 0,
        }
    }

    // Decode a registry DWORD. Any non-zero value is treated as Light (Windows only writes 0/1).
    pub fn from_dword(v: u32) -> Self {
        if v == 0 { Mode::Dark } else { Mode::Light }
    }

    // Human-readable label for `wtheme status` output.
    pub fn label(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }

    // Flip Light <-> Dark; used by the `toggle` subcommand.
    pub fn invert(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }
}
