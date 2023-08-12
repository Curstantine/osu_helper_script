use std::{env, ffi::OsString};
use std::{fs, io::Error};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let profile = match env::var("PROFILE").ok() {
        None => return Ok(()),
        Some(str) => str,
    };

    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => match profile.as_str() {
            "release" => {
                let path = OsString::from("./target/release/tab_completions");
                fs::create_dir_all(&path)?;
                path
            }
            _ => outdir,
        },
    };

    let mut cmd = Cli::command();
    let shell_types = vec![Shell::Fish, Shell::Bash, Shell::Zsh];

    for shell in shell_types {
        let path = generate_to(shell, &mut cmd, "osu_helper_script", &outdir)?;
        println!("cargo:note=completion file is generated: {path:?}");
    }

    Ok(())
}
