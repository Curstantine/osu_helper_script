use std::{cmp::Ordering, path::PathBuf};

use inquire::Confirm;

use crate::{github, local};

pub fn update(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    no_confirm: Option<bool>,
) -> anyhow::Result<()> {
    let no_confirm = no_confirm.unwrap_or(false);

    let installed_tags = local::get_local_release_tags(&install_dir)
        .expect("Couldn't get the installed versions from the local data directory");
    if installed_tags.is_empty() {
        panic!(
            "You don't have any known versions installed.\n\
            Use the install command to install a version."
        )
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
            println!("You're already on the latest version!");
            std::process::exit(0);
        }
        Ordering::Greater => {
            panic!(
                "LOL! You're on a newer version than the latest release!\n\
                Installed: {}\n\
                Latest: {}",
                installed_tags[0], &latest_release.tag_name
            )
        }
    }

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

    let app_image_asset = latest_release
        .get_app_image_asset()
        .expect("AppImage asset in missing from the release assets of this tag");

    let download_buffer = match local::download_release_asset(app_image_asset) {
        Ok(buffer) => buffer,
        Err(e) => panic!("Couldn't download the AppImage asset:\n{:#?}", e),
    };

    local::initialize_binary(
        &local_data_dir,
        &install_dir,
        &latest_release.tag_name,
        download_buffer,
    );

    local::remove_binary(&local_data_dir, &install_dir, latest_local_tag);
    local::update_desktop_database();

    println!("Successfully updated to {}!", &latest_release.tag_name);
    Ok(())
}
