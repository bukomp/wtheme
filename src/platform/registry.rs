// Registry read/write for the two Personalize DWORDs.

use anyhow::{Context, Result};
use windows_registry::Key;

use super::{APPS, SYSTEM};
use crate::mode::Mode;

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

// Small helper to keep the read path uncluttered.
fn read_dword(key: &Key, name: &str) -> Result<u32> {
    key.get_u32(name).with_context(|| format!("reading {name}"))
}

// Small helper to keep the write path uncluttered.
fn write_dword(key: &Key, name: &str, value: u32) -> Result<()> {
    key.set_u32(name, value).with_context(|| format!("writing {name}"))
}
