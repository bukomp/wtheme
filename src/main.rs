mod cli;
#[cfg(target_os = "windows")] mod mode;
#[cfg(target_os = "windows")] mod apply;
#[cfg(target_os = "windows")] mod platform;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

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
    use anyhow::Context;
    use cli::Cmd;
    use windows_registry::CURRENT_USER;

    let key = CURRENT_USER
        .create(platform::SUBKEY)
        .with_context(|| format!("opening HKCU\\{}", platform::SUBKEY))?;
    let (apps, sys) = platform::read_mode(&key)?;

    match cli.cmd {
        Cmd::Status => apply::print_status(apps, sys),
        Cmd::Dark | Cmd::Light | Cmd::Toggle => apply::apply_change(&cli, &key, apps, sys)?,
    }
    Ok(())
}
