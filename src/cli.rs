use crate::constants;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(
        long,
        default_value = constants::DEFAULT_INSTALL_DIR,
    )]
    /// "The base directory to install different versions of osu!"
    pub install_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a specified version of osu!
    Install { osu_version: Option<String> },
    /// Uninstall a specified version of osu!
    Uninstall,
    /// Update osu! to the latest version
    Update,
}
