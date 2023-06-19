use std::path::PathBuf;

use clap::Parser;
use cli::{Cli, Commands};

mod cli;
mod commands;
mod constants;
mod errors;
mod github;
mod local;
mod ureq;

fn main() {
    if !cfg!(target_os = "linux") {
        eprintln!("This program is only supported on Linux.");
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let local_data_dir = dirs::data_local_dir().expect("Couldn't find your local data directory.");
    let install_dir = match cli.install_dir {
        Some(string) => {
            let path = PathBuf::from(&string);
            if !path.exists() {
                eprintln!("The specified install directory does not exist.");
                std::process::exit(1);
            }
            if !path.is_dir() {
                eprintln!("The specified install directory is not a directory.");
                std::process::exit(1);
            }
            path
        }
        None => [local_data_dir.to_str().unwrap(), "games", "osu!"]
            .iter()
            .collect(),
    };

    let run = match cli.command {
        Commands::Install {
            osu_version: version,
        } => commands::install(local_data_dir, install_dir, version),
        Commands::Uninstall => unimplemented!(),
        Commands::Update { no_confirm } => {
            commands::update(local_data_dir, install_dir, no_confirm)
        }
    };

    run.unwrap();
}
