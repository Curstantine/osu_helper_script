use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long)]
    /// "The base directory to install different versions of osu!"
    pub install_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a specified version of osu!
    Install { osu_version: Option<String> },
    /// Uninstall a specified version of osu!
    Remove { osu_version: Option<String> },
    /// Update osu! to the latest version
    Update {
        /// Don't ask for confirmation before updating
        #[arg(long)]
        no_confirm: bool,
    },
}
