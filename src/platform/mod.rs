// Windows-only platform layer: registry I/O + WM_SETTINGCHANGE broadcast.

mod broadcast;
mod registry;

pub use broadcast::broadcast_setting_change;
pub use registry::{read_mode, write_mode};

// Documented path under HKCU that holds the user's light/dark preference.
pub const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
// DWORD value name controlling the per-app theme (Settings, Explorer chrome, etc.).
pub(crate) const APPS: &str = "AppsUseLightTheme";
// DWORD value name controlling the shell theme (taskbar, Start, notification flyouts).
pub(crate) const SYSTEM: &str = "SystemUsesLightTheme";
