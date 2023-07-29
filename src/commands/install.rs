use std::path::PathBuf;

use colored::*;
use inquire::Select;

use crate::{errors::Error, github, local};

pub fn install(local_data_dir: PathBuf, install_dir: PathBuf, version: Option<String>) -> Result<(), Error> {
    let installed_versions = local::get_local_release_tags(&install_dir)?;
    let release = match version {
        Some(version) => {
            let release = if version.to_lowercase() == "latest" {
                github::get_latest_release()
            } else {
                github::get_release(&version)
            };

            match release {
                Ok(rel) => rel,
                Err(e) => {
                    if let ureq::Error::Status(404, _) = *e {
                        return Err(Error::Descriptive(format!(
                            "Couldn't find a release with the tag {}",
                            version
                        )));
                    }

                    return Err(Error::from(e));
                }
            }
        }
        None => {
            let releases = github::get_releases()?;
            let release_tags = releases
                .iter()
                .map(|release| release.tag_name.clone())
                .filter(|tag| !installed_versions.contains(tag))
                .collect::<Vec<String>>();

            let selection = Select::new("Choose a version to download!", release_tags).prompt()?;
            releases
                .into_iter()
                .find(|release| release.tag_name == selection)
                .unwrap()
        }
    };

    local::initialize_binary(&local_data_dir, &install_dir, &release)?;

    println!("Successfully installed {}!", release.tag_name.green());

    Ok(())
}
