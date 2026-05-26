// Entry point for the `wtheme` CLI: parses args, opens the registry key once,
// then dispatches to either a status read or a write+broadcast.

mod cli;
#[cfg(target_os = "windows")] mod mode;
#[cfg(target_os = "windows")] mod apply;
#[cfg(target_os = "windows")] mod platform;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

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
    use anyhow::Context;
    use cli::Cmd;
    use windows_registry::CURRENT_USER;

    // Open HKCU\...\Personalize with read+write access (no admin needed).
    let key = CURRENT_USER
        .options()
        .read()
        .write()
        .open(platform::SUBKEY)
        .with_context(|| format!("opening HKCU\\{}", platform::SUBKEY))?;
    // Snapshot the current theme so `status` and `toggle` can both use it.
    let (apps, sys) = platform::read_mode(&key)?;

    match cli.cmd {
        // `status` is a pure read; print and return.
        Cmd::Status => apply::print_status(apps, sys),
        // The three mutating commands share the same write-and-broadcast path.
        Cmd::Dark | Cmd::Light | Cmd::Toggle => apply::apply_change(&cli, &key, apps, sys)?,
    }
    Ok(())
}
