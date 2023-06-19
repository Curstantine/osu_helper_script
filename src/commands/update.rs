use std::{cmp::Ordering, path::PathBuf};

use inquire::Confirm;

use crate::{github, local};

pub fn update(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    no_confirm: Option<bool>,
) -> anyhow::Result<()> {
    let no_confirm = no_confirm.unwrap_or(false);

    let installed_versions = local::get_local_release_tags(&install_dir)?;
    if installed_versions.is_empty() {
        panic!(
            "You don't have any known versions installed.\n\
            Use the install command to install a version."
        )
    }

    let latest_release = github::get_latest_release()?;
    let cmp = local::cmp_version_tag_ltr(&installed_versions[0], &latest_release.tag_name);

    if cmp == Ordering::Equal {
        println!("You're already on the latest version!");
        std::process::exit(0);
    }

    if cmp == Ordering::Greater {
        panic!(
            "You're on a newer version than the latest release!\n\
                Installed: {}\n\
                Latest: {}",
            installed_versions[0], latest_release.tag_name
        )
    }

    println!("An update is available!");
    println!("{} -> {}", installed_versions[0], latest_release.tag_name);

    if !no_confirm {
        let confirm = Confirm::new("Continue to install?")
            .with_default(true)
            .prompt()
            .unwrap();

        if !confirm {
            println!("Ok, aborting.");
            std::process::exit(0);
        }
    }

    unimplemented!()
}
