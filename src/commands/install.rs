use inquire::{error::InquireError, Select};
use std::path::PathBuf;

use crate::{errors::Error, github, local};

pub fn install(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    version: Option<String>,
) -> Result<(), Error> {
    let installed_versions = local::get_local_release_tags(&install_dir)?;
    let release = match version {
        Some(version) => {
            let release = if version.to_lowercase() == "latest" {
                github::get_latest_release()
            } else {
                github::get_release(&version)
            };

            if let Err(e) = release {
                if let ureq::Error::Status(404, _) = *e {
                    return Err(Error::Descriptive(format!(
                        "Couldn't find a release with the tag {}",
                        version
                    )));
                }

                return Err(Error::from(e));
            }

            release.unwrap()
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
                Err(e) => match e {
                    InquireError::OperationInterrupted | InquireError::OperationCanceled => {
                        return Err(Error::Abort)
                    }
                    InquireError::IO(io_error) => return Err(Error::Io(io_error)),
                    _ => panic!("Unhandled error: {:#?}", e),
                },
            }
        }
    };

    let app_image_asset = release
        .get_app_image_asset()
        .expect("AppImage asset in missing from the release assets of this tag");

    let download_buffer = local::download_release_asset(app_image_asset)?;

    local::initialize_binary(
        &local_data_dir,
        &install_dir,
        &release.tag_name,
        download_buffer,
    )?;

    println!("Successfully installed osu! {}!", release.tag_name);
    Ok(())
}
