use inquire::{Confirm, InquireError};
use std::{cmp::Ordering, path::PathBuf};

use crate::{errors::Error, github, local};

pub fn update(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    no_confirm: bool,
) -> Result<(), Error> {
    let installed_tags = local::get_local_release_tags(&install_dir)?;

    if installed_tags.is_empty() {
        return Err(Error::Descriptive(
            "You don't have any known versions installed.\nUse the install command to install a version."
                .to_string(),
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

    if !no_confirm {
        match Confirm::new("Continue to install?")
            .with_default(true)
            .prompt()
        {
            Ok(confirm) => {
                if !confirm {
                    return Err(Error::Abort);
                }
            }
            Err(e) => match e {
                InquireError::OperationInterrupted | InquireError::OperationCanceled => {
                    return Err(Error::Abort)
                }
                InquireError::IO(io_error) => return Err(Error::Io(io_error)),
                _ => panic!("Unhandled error: {:#?}", e),
            },
        };
    }

    let app_image_asset = latest_release
        .get_app_image_asset()
        .expect("AppImage asset in missing from the release assets of this tag");

    let download_buffer = local::download_release_asset(app_image_asset)?;

    local::initialize_binary(
        &local_data_dir,
        &install_dir,
        &latest_release.tag_name,
        download_buffer,
    )?;

    local::remove_binary(&local_data_dir, &install_dir, latest_local_tag)?;
    println!("Successfully updated to {}!", &latest_release.tag_name);

    Ok(())
}
