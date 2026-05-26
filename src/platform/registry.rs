use anyhow::{Context, Result};
use windows_registry::Key;

use super::{APPS, SYSTEM};
use crate::mode::Mode;

pub fn read_mode(key: &Key) -> Result<(Mode, Mode)> {
    let apps = read_dword(key, APPS)?;
    let sys = read_dword(key, SYSTEM)?;
    Ok((Mode::from_dword(apps), Mode::from_dword(sys)))
}

pub fn write_mode(key: &Key, apps: Option<Mode>, system: Option<Mode>) -> Result<()> {
    if let Some(m) = apps {
        write_dword(key, APPS, m.dword())?;
    }
    if let Some(m) = system {
        write_dword(key, SYSTEM, m.dword())?;
    }
    Ok(())
}

fn read_dword(key: &Key, name: &str) -> Result<u32> {
    key.get_u32(name).with_context(|| format!("reading {name}"))
}

fn write_dword(key: &Key, name: &str, value: u32) -> Result<()> {
    key.set_u32(name, value).with_context(|| format!("writing {name}"))
}
