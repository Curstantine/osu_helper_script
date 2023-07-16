use std::path::PathBuf;

use clap::Parser;
use cli::{Cli, Commands};
use errors::Error;

mod cli;
mod commands;
mod constants;
mod errors;
mod github;
mod local;
mod net;

fn main() -> errors::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(Error::Descriptive(
            "This program is only supported on Linux.".to_string(),
        ));
    }

    let cli = Cli::parse();

    let local_data_dir = dirs::data_local_dir().expect("Couldn't find your local data directory.");
    let install_dir = match cli.install_dir {
        Some(string) => {
            let path = PathBuf::from(&string);
            if !path.exists() {
                panic!("The specified install directory does not exist.");
            }
            if !path.is_dir() {
                panic!("The specified install directory is not a directory.");
            }
            path
        }
        None => [local_data_dir.to_str().unwrap(), "games", "osu!"].iter().collect(),
    };

    let run: errors::Result<()> = match cli.command {
        Commands::Install { osu_version } => commands::install(local_data_dir, install_dir, osu_version),
        Commands::Remove { osu_version } => commands::remove(local_data_dir, install_dir, osu_version),
        Commands::Update { no_confirm } => commands::update(local_data_dir, install_dir, no_confirm),
    };

    if let Err(e) = run {
        match e {
            Error::Abort => {}
            _ => eprintln!("{}", e),
        }
    };

    Ok(())
}
