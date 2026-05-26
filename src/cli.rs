// CLI surface: clap-derived `Cli` struct and `Cmd` subcommand enum.

use clap::{Parser, Subcommand};

// Top-level CLI definition (clap derive). One required subcommand + two global flags.
#[derive(Parser)]
#[command(name = "wtheme", version, about = "Switch Windows 11 light/dark mode")]
pub struct Cli {
    // The subcommand the user picked (dark/light/toggle/status).
    #[command(subcommand)]
    pub cmd: Cmd,

    // Only touch the apps theme value; mutually exclusive with --system-only.
    #[arg(long, global = true, conflicts_with = "system_only")]
    pub apps_only: bool,

    // Only touch the shell theme value (taskbar / Start).
    #[arg(long, global = true)]
    pub system_only: bool,
}

// The four supported actions.
#[derive(Subcommand, Copy, Clone)]
pub enum Cmd {
    Dark,
    Light,
    Toggle,
    Status,
}
