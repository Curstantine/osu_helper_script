use std::env::home_dir;

use clap::Parser;
use cli::{Cli, Commands};

mod cli;
mod commands;
mod constants;
mod github;
mod ureq;

fn main() {
    if !cfg!(target_os = "linux") {
        eprintln!("This program is only supported on Linux.");
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let install_dir = cli.install_dir.unwrap();
    let home_dir = match home_dir() {
        Some(home) => home,
        None => {
            eprintln!("Couldn't find your home directory.");
            std::process::exit(1);
        }
    };

    let run = match cli.command {
        Commands::Install {
            osu_version: version,
        } => commands::install(home_dir, install_dir, version),
        Commands::Uninstall => unimplemented!(),
        Commands::Update => unimplemented!(),
    };

    run.unwrap();
}
