use std::path::PathBuf;

use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use errors::Error;

mod cli;
mod commands;
mod constants;
mod errors;
mod github;
mod local;
mod net;

fn main() {
    if let Err(e) = run() {
        match e {
            Error::Abort => {}
            _ => eprintln!("{}", e),
        }
    };
}

fn run() -> errors::Result<()> {
    if !cfg!(target_family = "unix") {
        const WARN: &str = "This is intended to run on unix based systems.\n\
        If you are on Windows, you are better off using either the official installer or the Peppy.Osu! winget package.";
        println!("{}", WARN.red());

        const MESSAGE: &str = "I really want to break my system!";
        if !inquire::Confirm::new(MESSAGE).with_default(false).prompt()? {
            return Ok(());
        }
    } else if !cfg!(target_os = "linux") {
        const WARN: &str = "Support for non-linux systems is experimental. Expect things to break.";
        println!("{}", WARN.yellow());
    }

    let cli = Cli::parse();

    let local_data_dir = dirs::data_local_dir().expect("Couldn't find your local data directory.");
    let install_dir = match cli.install_dir {
        None => [local_data_dir.to_str().unwrap(), "games", "osu!"].iter().collect(),
        Some(string) => {
            let path = PathBuf::from(&string);
            if !path.try_exists()? {
                return Err(Error::Descriptive(
                    "The specified install directory does not exist.".to_owned(),
                ));
            }

            if !path.is_dir() {
                return Err(Error::Descriptive(
                    "The specified install directory is not a directory.".to_owned(),
                ));
            }

            path
        }
    };

    match cli.command {
        Commands::Install { osu_version } => commands::install(local_data_dir, install_dir, osu_version),
        Commands::Remove { osu_version } => commands::remove(local_data_dir, install_dir, osu_version),
        Commands::Update { no_confirm } => commands::update(local_data_dir, install_dir, no_confirm),
    }?;

    Ok(())
}
