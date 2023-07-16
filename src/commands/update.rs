use inquire::Confirm;
use std::{cmp::Ordering, path::PathBuf};

use crate::{errors::Error, github, local};

pub fn update(local_data_dir: PathBuf, install_dir: PathBuf, no_confirm: bool) -> Result<(), Error> {
    let installed_tags = local::get_local_release_tags(&install_dir)?;
    if installed_tags.is_empty() {
        return Err(Error::Descriptive(
            "You don't have any known versions installed.\nUse the install command to install a version.".to_owned(),
        ));
    }

    let latest_local_tag = &installed_tags[0];
    let latest_release = github::get_latest_release()?;

    match local::cmp_version_tag_ltr(latest_local_tag, &latest_release.tag_name) {
        Ordering::Less => {
            println!(
                "An update is available! {} -> {}",
                installed_tags[0], &latest_release.tag_name
            );
        }
        Ordering::Equal => {
            return Err(Error::Descriptive(format!(
                "You're already on the latest version: {}",
                &latest_release.tag_name
            )))
        }
        Ordering::Greater => {
            return Err(Error::Descriptive(format!(
                "LOL! You're on a newer version than the latest release!\n\
                Installed: {} Latest: {}",
                installed_tags[0], &latest_release.tag_name
            )))
        }
    }

    if !no_confirm && !Confirm::new("Continue to install?").with_default(true).prompt()? {
        return Err(Error::Abort);
    }

    local::initialize_binary(&local_data_dir, &install_dir, &latest_release)?;
    local::remove_binary(&local_data_dir, &install_dir, latest_local_tag)?;
    println!("Successfully updated to {}!", &latest_release.tag_name);

    Ok(())
}
