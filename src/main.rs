use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wtheme", version, about = "Switch Windows 11 light/dark mode")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    /// Only change the apps theme (AppsUseLightTheme).
    #[arg(long, global = true, conflicts_with = "system_only")]
    apps_only: bool,

    /// Only change the system/shell theme (SystemUsesLightTheme).
    #[arg(long, global = true)]
    system_only: bool,

    /// Skip the WM_SETTINGCHANGE broadcast (no live refresh).
    #[arg(long, global = true)]
    no_broadcast: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Set dark mode.
    Dark,
    /// Set light mode.
    Light,
    /// Invert the current mode.
    Toggle,
    /// Print the current theme values.
    Status,
}

#[cfg(target_os = "windows")]
#[derive(Copy, Clone, Eq, PartialEq)]
enum Mode {
    Light,
    Dark,
}

#[cfg(target_os = "windows")]
impl Mode {
    fn dword(self) -> u32 {
        match self {
            Mode::Light => 1,
            Mode::Dark => 0,
        }
    }
    fn from_dword(v: u32) -> Self {
        if v == 0 { Mode::Dark } else { Mode::Light }
    }
    fn label(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e:#}");
            1
        }
    };
    std::process::exit(code);
}

#[cfg(not(target_os = "windows"))]
fn run(_cli: Cli) -> Result<()> {
    anyhow::bail!("wtheme only runs on Windows (target_os != \"windows\")");
}

#[cfg(target_os = "windows")]
fn run(cli: Cli) -> Result<()> {
    use platform::{broadcast_setting_change, open_personalize, read_mode, write_mode};

    let key = open_personalize()?;
    let (cur_apps, cur_sys) = read_mode(&key)?;

    match cli.cmd {
        Cmd::Status => {
            println!("apps:   {}", cur_apps.label());
            println!("system: {}", cur_sys.label());
            return Ok(());
        }
        Cmd::Dark | Cmd::Light | Cmd::Toggle => {
            let (target_apps, target_sys) = match cli.cmd {
                Cmd::Dark => (Mode::Dark, Mode::Dark),
                Cmd::Light => (Mode::Light, Mode::Light),
                Cmd::Toggle => {
                    let invert = |m: Mode| if m == Mode::Light { Mode::Dark } else { Mode::Light };
                    (invert(cur_apps), invert(cur_sys))
                }
                _ => unreachable!(),
            };

            let write_apps = !cli.system_only;
            let write_sys = !cli.apps_only;

            write_mode(&key, write_apps.then_some(target_apps), write_sys.then_some(target_sys))?;

            if !cli.no_broadcast {
                broadcast_setting_change()?;
            }

            let (new_apps, new_sys) = read_mode(&key)?;
            println!("apps:   {}", new_apps.label());
            println!("system: {}", new_sys.label());
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
mod platform {
    use super::Mode;
    use anyhow::{Context, Result};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    use windows_registry::{Key, CURRENT_USER};

    const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS: &str = "AppsUseLightTheme";
    const SYSTEM: &str = "SystemUsesLightTheme";

    pub fn open_personalize() -> Result<Key> {
        CURRENT_USER
            .options()
            .read()
            .write()
            .open(SUBKEY)
            .with_context(|| format!("opening HKCU\\{SUBKEY}"))
    }

    pub fn read_mode(key: &Key) -> Result<(Mode, Mode)> {
        let apps = key.get_u32(APPS).with_context(|| format!("reading {APPS}"))?;
        let sys = key.get_u32(SYSTEM).with_context(|| format!("reading {SYSTEM}"))?;
        Ok((Mode::from_dword(apps), Mode::from_dword(sys)))
    }

    pub fn write_mode(key: &Key, apps: Option<Mode>, system: Option<Mode>) -> Result<()> {
        if let Some(m) = apps {
            key.set_u32(APPS, m.dword()).with_context(|| format!("writing {APPS}"))?;
        }
        if let Some(m) = system {
            key.set_u32(SYSTEM, m.dword()).with_context(|| format!("writing {SYSTEM}"))?;
        }
        Ok(())
    }

    pub fn broadcast_setting_change() -> Result<()> {
        let param: Vec<u16> = "ImmersiveColorSet\0".encode_utf16().collect();
        let mut result: usize = 0;
        unsafe {
            SendMessageTimeoutW(
                HWND(HWND_BROADCAST.0),
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(PCWSTR(param.as_ptr()).0 as isize),
                SMTO_ABORTIFHUNG,
                1000,
                Some(&mut result as *mut usize as *mut _),
            );
        }
        Ok(())
    }
}
