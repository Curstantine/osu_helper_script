use inquire::Select;
use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

use crate::{
    constants::{TEMP_DIR, USER_AGENT},
    github,
    local::{self, download_release_asset, initialize_binary},
    ureq::{box_request, download_file_with_progress},
};

pub fn install(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    version: Option<String>,
) -> anyhow::Result<()> {
    let installed_versions = local::get_local_release_tags(&install_dir)?;
    let version = match version {
        Some(version) => {
            let en = if version.to_lowercase() == "latest" {
                github::get_latest_release()
            } else {
                github::get_release(&version)
            };

            if let Err(e) = en {
                match *e {
                    ureq::Error::Status(404, _) => {
                        eprintln!("Couldn't find a release with the tag \"{}\"", version);
                        std::process::exit(1);
                    }
                    e => {
                        eprintln!("Came across an error while trying to find: {:#?}", e);
                        std::process::exit(1);
                    }
                }
            }

            en.unwrap()
        }
        None => {
            let releases = github::get_releases()?;
            let release_tags = releases
                .iter()
                .map(|release| release.tag_name.clone())
                .filter(|tag| !installed_versions.contains(tag))
                .collect::<Vec<String>>();

            match Select::new("Choose a version to download!", release_tags).prompt() {
                Ok(selection) => releases
                    .into_iter()
                    .find(|release| release.tag_name == selection)
                    .unwrap(),
                Err(e) => {
                    eprintln!("Couldn't resolve a version from the input: {:#?}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    let app_image_asset = match version
        .assets
        .into_iter()
        .find(|asset| asset.name.ends_with(".AppImage"))
    {
        Some(asset) => asset,
        None => {
            eprintln!("Couldn't find an AppImage asset in the release.");
            std::process::exit(1);
        }
    };

    let download_buffer = match download_release_asset(&app_image_asset) {
        Ok(buffer) => buffer,
        Err(e) => panic!("Couldn't download the AppImage asset:\n {:#?}", e),
    };

    initialize_binary(
        &local_data_dir,
        &install_dir,
        &version.tag_name,
        download_buffer,
    );

    println!("Successfully installed osu! {}!", version.tag_name);

    Ok(())
}
