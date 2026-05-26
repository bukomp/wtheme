// Mutation path: compute target modes, write, broadcast, re-read, print.

use anyhow::Result;

use crate::cli::{Cli, Cmd};
use crate::mode::Mode;
use crate::platform;

// Pretty-print the current theme values.
pub fn print_status(apps: Mode, sys: Mode) {
    println!("apps:   {}", apps.label());
    println!("system: {}", sys.label());
}

// Compute target modes from the subcommand and current state.
// `cur_*` only matters for `Toggle`; Dark/Light ignore it.
fn target_modes(cmd: Cmd, cur_apps: Mode, cur_sys: Mode) -> (Mode, Mode) {
    match cmd {
        Cmd::Dark => (Mode::Dark, Mode::Dark),
        Cmd::Light => (Mode::Light, Mode::Light),
        Cmd::Toggle => (cur_apps.invert(), cur_sys.invert()),
        // `Status` doesn't mutate, so it never reaches here.
        Cmd::Status => unreachable!("status is handled before target_modes"),
    }
}

// Write the requested values, broadcast, then print the new state.
pub fn apply_change(
    cli: &Cli,
    key: &windows_registry::Key,
    cur_apps: Mode,
    cur_sys: Mode,
) -> Result<()> {
    // Resolve which Mode each registry value should end up as.
    let (target_apps, target_sys) = target_modes(cli.cmd, cur_apps, cur_sys);

    // --system-only suppresses the apps write; --apps-only suppresses the system write.
    let apps_to_write = (!cli.system_only).then_some(target_apps);
    let sys_to_write = (!cli.apps_only).then_some(target_sys);

    // Write whichever values are still Some(..).
    platform::write_mode(key, apps_to_write, sys_to_write)?;

    // Tell running apps to repaint immediately.
    platform::broadcast_setting_change()?;

    // Re-read so the printed state reflects what actually got written.
    let (apps, sys) = platform::read_mode(key)?;
    print_status(apps, sys);
    Ok(())
}
