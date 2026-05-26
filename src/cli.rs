use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wtheme", version, about = "Switch Windows 11 light/dark mode")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,

    #[arg(long, global = true, conflicts_with = "system_only")]
    pub apps_only: bool,

    #[arg(long, global = true)]
    pub system_only: bool,
}

#[derive(Subcommand, Copy, Clone)]
pub enum Cmd {
    Dark,
    Light,
    Toggle,
    Status,
}
