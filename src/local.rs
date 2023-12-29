use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::errors::{self, ignore_io_not_found, Error};
use crate::github::{self, GithubRelease};
use crate::net;

/// Lists all the releases available in the install_dir.
///
/// Returned vector is sorted in descending order.
///
/// ## NOTE
///
/// Do not rely on this function to check whether install_dir exists or not.
/// For cases where install_dir scan return [std::io::ErrorKind::NotFound], this function will return an empty vector.
pub fn get_local_release_tags(install_dir: &Path) -> io::Result<Vec<String>> {
    let release_tags: Vec<String> = match fs::read_dir(install_dir) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                if path.is_dir() {
                    return None;
                }

                let name = path.file_name().map(|name| name.to_string_lossy().to_string())?;
                if name.ends_with(".AppImage") {
                    Some(name.replace(".AppImage", ""))
                } else {
                    None
                }
            })
            .collect(),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(Vec::with_capacity(0));
            }

            return Err(e);
        }
    };

    Ok(sort_version_tags_desc(release_tags))
}

/// Sorts a vector of version tags in descending order.
///
/// 2023.617.1 > 2023.617.0 > 2023.612.0
pub fn sort_version_tags_desc(mut tags: Vec<String>) -> Vec<String> {
    tags.sort_by(|a, b| cmp_version_tag_ltr(a, b).reverse());
    tags
}

/// Compares whether left is greater than right.
pub fn cmp_version_tag_ltr(left: &str, right: &str) -> std::cmp::Ordering {
    let left = left.split('.').collect::<Vec<&str>>();
    let right = right.split('.').collect::<Vec<&str>>();

    let left = left.iter().map(|s| s.parse::<u32>().unwrap_or(0)).collect::<Vec<u32>>();
    let right = right
        .iter()
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect::<Vec<u32>>();

    let left = left
        .iter()
        .enumerate()
        .map(|(i, n)| n * 10u32.pow((left.len() - i - 1) as u32))
        .sum::<u32>();
    let right = right
        .iter()
        .enumerate()
        .map(|(i, n)| n * 10u32.pow((right.len() - i - 1) as u32))
        .sum::<u32>();

    left.cmp(&right)
}

// TODO: Add support for other os alternatives.
fn update_desktop_database(local_data_dir: &Path) -> errors::Result<()> {
    if cfg!(target_os = "linux") {
        print!("Updating the desktop database...");

        let desktop_dir = local_data_dir.join("applications").canonicalize().unwrap();
        let output = std::process::Command::new("update-desktop-database")
            .arg(desktop_dir.to_str().unwrap())
            .output()?;

        if !output.status.success() {
            return Err(Error::Descriptive(format!(
                "Failed to update the desktop database:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        println!("\rSuccessfully updated the desktop database!");
    } else {
        println!("Desktop database update is only supported on Linux for now.");
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn set_permission_as_executable(file: &Path) -> errors::Result<()> {
    use std::fs::Permissions;
    use std::os::unix::prelude::PermissionsExt;

    fs::set_permissions(file, Permissions::from_mode(0o755)).map_err(|e| Error::Io {
        source: e,
        context: Some(file.to_str().unwrap().to_owned()),
    })?;

    Ok(())
}

fn create_desktop_entry(name: &str, icon_dir: &Path, exec_path: &Path, entry_path: &Path) -> errors::Result<()> {
    if cfg!(target_os = "linux") {
        let desktop_entry_content = format!(
            "[Desktop Entry]\n\
        Name={name}\n\
        Icon={icon_dir}\n\
        Comment=rhythm is just a *click* away!\n\
        Exec={exec_dir}\n\
        Version=1.0\n\
        Type=Application\n\
        Categories=Game;",
            icon_dir = icon_dir.canonicalize()?.to_str().unwrap(),
            exec_dir = exec_path.canonicalize()?.to_str().unwrap(),
        );

        print!("Creating the desktop entry...");
        fs::write(entry_path, desktop_entry_content).map_err(|e| Error::Io {
            source: e,
            context: Some(entry_path.to_str().unwrap().to_owned()),
        })?;

        println!("\rSuccessfully created the desktop entry at {}!", entry_path.display());
    } else {
        println!("Desktop entry creation is only supported on Linux for now.");
    }

    Ok(())
}

/// Initializes all prerequisites required to move the [download_buffer] into a
/// file and creates the desktop entry.
pub fn initialize_binary(local_data_dir: &Path, install_dir: &Path, release: &GithubRelease) -> errors::Result<()> {
    let install_data = InstallData::new(local_data_dir, install_dir, &release.tag_name);
    let source_icon_path = install_dir.join("osu.png");

    if !install_dir.try_exists()? {
        fs::create_dir_all(install_dir).map_err(|e| Error::Io {
            source: e,
            context: Some(install_dir.to_str().unwrap().to_owned()),
        })?;
    }

    if install_data.install_path.try_exists()? {
        // TODO check sizes are the same;
        println!("Found a previous binary of this release, skipping download");
    } else {
        let app_image_asset = release
            .get_app_image_asset()
            .expect("AppImage asset in missing from the release assets of this tag");
        let download_buffer = net::download_release_asset(app_image_asset)?;
        // let download_buffer = vec![0; 0];

        fs::write(&install_data.install_path, download_buffer).map_err(|e| Error::Io {
            source: e,
            context: Some(install_data.install_path.to_str().unwrap().to_owned()),
        })?;
    }

    #[cfg(target_family = "unix")]
    set_permission_as_executable(&install_data.install_path)?;

    if !source_icon_path.try_exists()? {
        let icon_data = github::get_icon()?;
        fs::write(&source_icon_path, icon_data)?;
    }

    create_desktop_entry(
        format!("osu! {version}", version = &release.tag_name).as_str(),
        &source_icon_path,
        &install_data.install_path,
        &install_data.desktop_entry_path,
    )?;

    update_desktop_database(local_data_dir)?;

    Ok(())
}

/// Removes the binary and the desktop entry from their respective directories.
///
/// NOTE: This function internally handles all the errors and events, so
/// there's no need to handle them externally.
pub fn remove_binary(local_data_dir: &Path, install_dir: &Path, tag_name: &str) -> errors::Result<()> {
    let install_data = InstallData::new(local_data_dir, install_dir, tag_name);

    print!("Removing the {} binary...", tag_name);
    ignore_io_not_found(
        fs::remove_file(&install_data.install_path),
        format!("Successfully removed the {} binary.", tag_name),
        format!("Couldn't find the {} binary, skipping...", tag_name),
    )?;

    print!("Removing the {} desktop entry...", tag_name);
    ignore_io_not_found(
        fs::remove_file(&install_data.desktop_entry_path),
        format!("Successfully removed the {} desktop entry.", tag_name),
        format!("Couldn't find the {} desktop entry, skipping...", tag_name),
    )?;

    update_desktop_database(local_data_dir)?;

    Ok(())
}

#[derive(Debug)]
/// Contains common paths and file names required to manipulate a single binary.
struct InstallData {
    pub desktop_entry_path: PathBuf,
    pub install_path: PathBuf,
}

impl InstallData {
    fn new(local_data_dir: &Path, install_dir: &Path, release_tag_name: &str) -> Self {
        let desktop_dir = local_data_dir.join("applications");
        let app_image_file_name = format!("{}.AppImage", release_tag_name);
        let desktop_file_name = format!("osu!-{}.desktop", release_tag_name);

        Self {
            install_path: install_dir.join(app_image_file_name),
            desktop_entry_path: desktop_dir.join(desktop_file_name),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn version_tag_cmp_works() {
        assert_eq!(
            super::cmp_version_tag_ltr("2023.617.1", "2023.617.0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            super::cmp_version_tag_ltr("2023.617.0", "2023.617.1"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            super::cmp_version_tag_ltr("2023.617.0", "2023.617.0"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn desc_sort_works() {
        let versions = vec![
            String::from("2023.617.0"),
            String::from("2023.612.0"),
            String::from("2022.142.1"),
            String::from("2023.612.1"),
        ];

        let sorted = super::sort_version_tags_desc(versions);
        assert_eq!(
            sorted,
            vec![
                String::from("2023.617.0"),
                String::from("2023.612.1"),
                String::from("2023.612.0"),
                String::from("2022.142.1"),
            ]
        )
    }

    #[test]
    fn test_install_data() {
        let local_data_dir = Path::new("/home/username/.local/share");
        let install_dir = local_data_dir.join("games/osu!");
        let release_tag_name = String::from("2023.617.0");

        let install_data = super::InstallData::new(local_data_dir, &install_dir, &release_tag_name);

        assert_eq!(
            install_data.install_path,
            Path::new("/home/username/.local/share/games/osu!/2023.617.0.AppImage")
        );
        assert_eq!(
            install_data.desktop_entry_path,
            Path::new("/home/username/.local/share/applications/osu!-2023.617.0.desktop")
        );
    }
}
