use anyhow::Result;

use crate::cli::{Cli, Cmd};
use crate::mode::Mode;
use crate::platform;

pub fn print_status(apps: Mode, sys: Mode) {
    println!("apps:   {}", apps.label());
    println!("system: {}", sys.label());
}

fn target_modes(cmd: Cmd, cur_apps: Mode, cur_sys: Mode) -> (Mode, Mode) {
    match cmd {
        Cmd::Dark => (Mode::Dark, Mode::Dark),
        Cmd::Light => (Mode::Light, Mode::Light),
        Cmd::Toggle => (cur_apps.invert(), cur_sys.invert()),
        Cmd::Status => unreachable!("status is handled before target_modes"),
    }
}

pub fn apply_change(
    cli: &Cli,
    key: &windows_registry::Key,
    cur_apps: Mode,
    cur_sys: Mode,
) -> Result<()> {
    let (target_apps, target_sys) = target_modes(cli.cmd, cur_apps, cur_sys);

    let apps_to_write = (!cli.system_only).then_some(target_apps);
    let sys_to_write = (!cli.apps_only).then_some(target_sys);

    platform::write_mode(key, apps_to_write, sys_to_write)?;
    platform::broadcast_setting_change()?;

    let (apps, sys) = platform::read_mode(key)?;
    print_status(apps, sys);
    Ok(())
}
