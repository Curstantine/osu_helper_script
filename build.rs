use std::env;
use std::io::Error;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    let shell_types = vec![Shell::Fish, Shell::Bash, Shell::Zsh];

    for shell in shell_types {
        let path = generate_to(shell, &mut cmd, "osu_helper_script", &outdir)?;
        println!("cargo:note=completion file is generated: {path:?}");
    }

    Ok(())
}
