// Entry point for the `wtheme` CLI: parses args and dispatches to the Windows-only
// implementation. On non-Windows hosts the binary still compiles, but `run` errors out.

use anyhow::Result;
use clap::{Parser, Subcommand};

// Top-level CLI definition (clap derive). One required subcommand + three global flags.
#[derive(Parser)]
#[command(name = "wtheme", version, about = "Switch Windows 11 light/dark mode")]
struct Cli {
    // The subcommand the user picked (dark/light/toggle/status).
    #[command(subcommand)]
    cmd: Cmd,

    // Only touch the apps theme value; mutually exclusive with --system-only.
    #[arg(long, global = true, conflicts_with = "system_only")]
    apps_only: bool,

    // Only touch the shell theme value (taskbar / Start).
    #[arg(long, global = true)]
    system_only: bool,

    // Skip the WM_SETTINGCHANGE broadcast (keys still get written; running apps just won't repaint live).
    #[arg(long, global = true)]
    no_broadcast: bool,
}

// The four supported actions.
#[derive(Subcommand, Copy, Clone)]
enum Cmd {
    Dark,
    Light,
    Toggle,
    Status,
}

// Two-state theme value. Stored in the registry as a DWORD: 0 = Dark, 1 = Light.
#[cfg(target_os = "windows")]
#[derive(Copy, Clone, Eq, PartialEq)]
enum Mode {
    Light,
    Dark,
}

#[cfg(target_os = "windows")]
impl Mode {
    // Encode for the registry: matches Windows' AppsUseLightTheme / SystemUsesLightTheme convention.
    fn dword(self) -> u32 {
        match self {
            Mode::Light => 1,
            Mode::Dark => 0,
        }
    }

    // Decode a registry DWORD. Any non-zero value is treated as Light (Windows only writes 0/1).
    fn from_dword(v: u32) -> Self {
        if v == 0 { Mode::Dark } else { Mode::Light }
    }

    // Human-readable label for `wtheme status` output.
    fn label(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }

    // Flip Light <-> Dark; used by the `toggle` subcommand.
    #[cfg(target_os = "windows")]
    fn invert(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }
}

// Process entry point: parse args, run, map errors to an exit code.
fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            // `{:#}` prints the full anyhow context chain.
            eprintln!("error: {e:#}");
            1
        }
    };
    std::process::exit(code);
}

// Non-Windows stub so the crate still compiles on macOS/Linux for sanity checks.
#[cfg(not(target_os = "windows"))]
fn run(_cli: Cli) -> Result<()> {
    anyhow::bail!("wtheme only runs on Windows (target_os != \"windows\")");
}

// Windows entry point: open the key once, then dispatch by subcommand.
#[cfg(target_os = "windows")]
fn run(cli: Cli) -> Result<()> {
    // Open HKCU\...\Personalize with read+write access (no admin needed).
    let key = platform::open_personalize()?;
    // Snapshot the current theme so `status` and `toggle` can both use it.
    let (apps, sys) = platform::read_mode(&key)?;

    match cli.cmd {
        // `status` is a pure read; print and return.
        Cmd::Status => print_status(apps, sys),
        // The three mutating commands share the same write-and-broadcast path.
        Cmd::Dark | Cmd::Light | Cmd::Toggle => apply_change(&cli, &key, apps, sys)?,
    }
    Ok(())
}

// Pretty-print the current theme values.
#[cfg(target_os = "windows")]
fn print_status(apps: Mode, sys: Mode) {
    println!("apps:   {}", apps.label());
    println!("system: {}", sys.label());
}

// Compute target modes from the subcommand and current state.
// `cur_*` only matters for `Toggle`; Dark/Light ignore it.
#[cfg(target_os = "windows")]
fn target_modes(cmd: Cmd, cur_apps: Mode, cur_sys: Mode) -> (Mode, Mode) {
    match cmd {
        Cmd::Dark => (Mode::Dark, Mode::Dark),
        Cmd::Light => (Mode::Light, Mode::Light),
        Cmd::Toggle => (cur_apps.invert(), cur_sys.invert()),
        // `Status` doesn't mutate, so it never reaches here.
        Cmd::Status => unreachable!("status is handled before target_modes"),
    }
}

// Write the requested values, optionally broadcast, then print the new state.
#[cfg(target_os = "windows")]
fn apply_change(cli: &Cli, key: &windows_registry::Key, cur_apps: Mode, cur_sys: Mode) -> Result<()> {
    // Resolve which Mode each registry value should end up as.
    let (target_apps, target_sys) = target_modes(cli.cmd, cur_apps, cur_sys);

    // --system-only suppresses the apps write; --apps-only suppresses the system write.
    let apps_to_write = (!cli.system_only).then_some(target_apps);
    let sys_to_write = (!cli.apps_only).then_some(target_sys);

    // Write whichever values are still Some(..).
    platform::write_mode(key, apps_to_write, sys_to_write)?;

    // Tell running apps to repaint. Skipped when the user passed --no-broadcast.
    if !cli.no_broadcast {
        platform::broadcast_setting_change()?;
    }

    // Re-read so the printed state reflects what actually got written.
    let (apps, sys) = platform::read_mode(key)?;
    print_status(apps, sys);
    Ok(())
}

// All Win32 / registry interaction lives behind this module so `run` stays platform-light.
#[cfg(target_os = "windows")]
mod platform {
    use super::Mode;
    use anyhow::{Context, Result};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    pub use windows_registry::Key;
    use windows_registry::CURRENT_USER;

    // Documented path under HKCU that holds the user's light/dark preference.
    const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    // DWORD value name controlling the per-app theme (Settings, Explorer chrome, etc.).
    const APPS: &str = "AppsUseLightTheme";
    // DWORD value name controlling the shell theme (taskbar, Start, notification flyouts).
    const SYSTEM: &str = "SystemUsesLightTheme";
    // Single, short timeout for the broadcast — we don't want one hung window to stall the CLI.
    const BROADCAST_TIMEOUT_MS: u32 = 1000;
    // Null-terminated UTF-16 string passed as lParam to identify the changed setting.
    const IMMERSIVE_COLOR_SET: &str = "ImmersiveColorSet\0";

    // Open the Personalize subkey with read+write rights. HKCU never needs elevation.
    pub fn open_personalize() -> Result<Key> {
        CURRENT_USER
            .options()
            .read()
            .write()
            .open(SUBKEY)
            .with_context(|| format!("opening HKCU\\{SUBKEY}"))
    }

    // Read both DWORDs and return them as decoded Modes.
    pub fn read_mode(key: &Key) -> Result<(Mode, Mode)> {
        let apps = read_dword(key, APPS)?;
        let sys = read_dword(key, SYSTEM)?;
        Ok((Mode::from_dword(apps), Mode::from_dword(sys)))
    }

    // Conditionally write each value; passing None leaves the existing registry entry untouched.
    pub fn write_mode(key: &Key, apps: Option<Mode>, system: Option<Mode>) -> Result<()> {
        if let Some(m) = apps {
            write_dword(key, APPS, m.dword())?;
        }
        if let Some(m) = system {
            write_dword(key, SYSTEM, m.dword())?;
        }
        Ok(())
    }

    // Broadcast WM_SETTINGCHANGE with lParam="ImmersiveColorSet" — same notification the
    // Settings app sends. Listening apps (Explorer, Edge, modern UWP shells) repaint immediately.
    pub fn broadcast_setting_change() -> Result<()> {
        // Encode the section name as a null-terminated wide string for lParam.
        let param: Vec<u16> = IMMERSIVE_COLOR_SET.encode_utf16().collect();
        // SendMessageTimeoutW writes the each recipient's reply here; we ignore the value.
        let mut result: usize = 0;
        // Safety: `param` outlives the call; `result` is a valid &mut for the duration.
        unsafe {
            SendMessageTimeoutW(
                HWND(HWND_BROADCAST.0),                                  // deliver to every top-level window
                WM_SETTINGCHANGE,                                        // the message Windows uses for setting changes
                WPARAM(0),                                               // wParam is unused for this notification
                LPARAM(PCWSTR(param.as_ptr()).0 as isize),               // lParam = pointer to "ImmersiveColorSet"
                SMTO_ABORTIFHUNG,                                        // skip windows that are hung instead of blocking
                BROADCAST_TIMEOUT_MS,                                    // per-window timeout
                Some(&mut result as *mut usize as *mut _),               // out-param we don't consume
            );
        }
        Ok(())
    }

    // Small helper to keep the read path uncluttered.
    fn read_dword(key: &Key, name: &str) -> Result<u32> {
        key.get_u32(name).with_context(|| format!("reading {name}"))
    }

    // Small helper to keep the write path uncluttered.
    fn write_dword(key: &Key, name: &str, value: u32) -> Result<()> {
        key.set_u32(name, value).with_context(|| format!("writing {name}"))
    }
}
