use std::path::PathBuf;

use inquire::{Confirm, Select};

use crate::{
    errors::{Error, Result},
    local,
};

pub fn remove(local_data_dir: PathBuf, install_dir: PathBuf, version: Option<String>) -> Result<()> {
    let installed_tags = local::get_local_release_tags(&install_dir)?;
    if installed_tags.is_empty() {
        return Err(Error::Descriptive(
            "You don't have any known versions installed.\nUse the install command to install a version.".to_owned(),
        ));
    }

    let version_tag = match version {
        Some(version) => {
            if !installed_tags.contains(&version) {
                let message = format!("Couldn't find a release with the tag {}", version);
                return Err(Error::Descriptive(message));
            }

            version
        }
        None => {
            let mut selection = installed_tags.clone();
            selection.push("All".to_owned());
            Select::new("Choose a version to remove!", selection).prompt()?
        }
    };

    let confirm_etc_delete = || -> Result<()> {
        let message = format!(
            "Do you want to remove the icon and other files as well? THIS WILL DELETE {}",
            install_dir.display()
        );

        if Confirm::new(&message).prompt()? {
            std::fs::remove_dir_all(&install_dir)?;
        }

        Ok(())
    };

    {
        let message = format!("Are you sure you want to delete all {} versions?", installed_tags.len());
        if version_tag == "All" && Confirm::new(&message).prompt()? {
            for tag in installed_tags {
                local::remove_binary(&local_data_dir, &install_dir, &tag)?;
            }

            confirm_etc_delete()?;
            return Ok(());
        }
    }

    {
        let message = format!("Are you sure you want to delete {}?", version_tag);
        if Confirm::new(&message).prompt()? {
            local::remove_binary(&local_data_dir, &install_dir, &version_tag)?;

            if installed_tags.len() == 1 {
                confirm_etc_delete()?;
            }

            return Ok(());
        }
    }

    Err(Error::Abort)
}
