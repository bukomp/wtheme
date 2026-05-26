mod broadcast;
mod registry;

pub use broadcast::broadcast_setting_change;
pub use registry::{read_mode, write_mode};

pub const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
pub(crate) const APPS: &str = "AppsUseLightTheme";
pub(crate) const SYSTEM: &str = "SystemUsesLightTheme";
