use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::errors::{self, Error};
use crate::github::GithubRelease;
use crate::local;
use crate::{
    constants::{TEMP_DIR, USER_AGENT},
    github::{self, GithubReleaseAsset},
    net,
};

/// Lists all the releases available in the install_dir.
///
/// Returned vector is sorted in descending order.
pub fn get_local_release_tags(install_dir: &Path) -> io::Result<Vec<String>> {
    let release_tags = fs::read_dir(install_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_dir() {
                return None;
            }

            let file_same = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())?;

            if file_same.ends_with(".AppImage") {
                Some(file_same.replace(".AppImage", ""))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

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

    let left = left
        .iter()
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect::<Vec<u32>>();
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

/// Downloads a given release asset with a progress bar.
///
/// Internally, this requests the asset, and then streams the response into a Vec<u8>.
pub fn download_release_asset(asset: &GithubReleaseAsset) -> errors::Result<Vec<u8>> {
    let response = net::box_request(
        ureq::get(&asset.browser_download_url)
            .set("Accept", "application/octet-stream")
            .set("User-Agent", USER_AGENT),
    )?;

    let server_size = match response.header("Content-Length").unwrap().parse::<u64>() {
        Ok(size) => size,
        Err(_) => return Err(Error::Descriptive("Couldn't parse icon size".into())),
    };

    if server_size != asset.size {
        return Err(Error::Descriptive(format!(
            "The file size of the downloadable file doesn't match the size of the asset on GitHub. ({} != {})",
            server_size, asset.size
        )));
    }

    Ok(net::download_file_with_progress(
        response.into_reader(),
        server_size,
    )?)
}

pub fn update_desktop_database(local_data_dir: &Path) -> errors::Result<()> {
    let desktop_dir = local_data_dir.join("applications").canonicalize().unwrap();
    let output = std::process::Command::new("update-desktop-database")
        .arg(desktop_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute update-desktop-database");

    if !output.status.success() {
        return Err(Error::Descriptive(format!(
            "Failed to update the desktop database:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Initializes all prerequisites required to move the [download_buffer] into a
/// file and creates the desktop entry.
pub fn initialize_binary(
    local_data_dir: &Path,
    install_dir: &Path,
    release: &GithubRelease,
) -> errors::Result<()> {
    let app_image_asset = release
        .get_app_image_asset()
        .expect("AppImage asset in missing from the release assets of this tag");
    let download_buffer = local::download_release_asset(app_image_asset)?;

    let install_data = InstallData::new(local_data_dir, install_dir, &release.tag_name);
    let tmp_file_path = install_data.get_temp_file_path();
    let source_icon_path = install_dir.join("osu.png");

    fs::create_dir_all(TEMP_DIR)?;
    fs::write(&tmp_file_path, download_buffer)?;

    if !install_dir.exists() {
        fs::create_dir_all(install_dir)?;
    }

    fs::rename(tmp_file_path, &install_data.install_path)?;
    fs::set_permissions(&install_data.install_path, Permissions::from_mode(0o755))?;
    println!("Moved {} to {:#?}", &release.tag_name, install_dir);

    if !source_icon_path.exists() {
        github::get_icon()?;
    }

    let desktop_entry_content = format!(
        "[Desktop Entry]\n\
        Name=osu! {version}\n\
        Icon={icon_dir}\n\
        Comment=rhythm is just a *click* away!\n\
        Exec={exec_dir}\n\
        Version=1.0\n\
        Type=Application\n\
        Categories=Game;",
        version = &release.tag_name,
        icon_dir = source_icon_path.canonicalize().unwrap().to_str().unwrap(),
        exec_dir = install_data
            .install_path
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    println!("Creating the desktop entry...");
    fs::write(&install_data.desktop_entry_path, desktop_entry_content)?;
    println!(
        "\rSuccessfully created the desktop entry at {}!",
        &install_data.desktop_entry_path.to_str().unwrap()
    );

    println!("Cleaning up temporary files...");
    fs::remove_dir_all(TEMP_DIR)?;
    println!("\rSuccessfully cleaned up temporary files!");

    println!("Updating the desktop database...");
    update_desktop_database(local_data_dir)?;
    println!("\rSuccessfully updated the desktop database!");

    Ok(())
}

/// Removes the binary and the desktop entry from their respective directories.
///
/// NOTE: This function internally handles all the errors and events, so
/// there's no need to handle them externally.
pub fn remove_binary(
    local_data_dir: &Path,
    install_dir: &Path,
    tag_name: &str,
) -> errors::Result<()> {
    let install_data = InstallData::new(local_data_dir, install_dir, tag_name);

    println!("Removing the {} binary...", tag_name);
    fs::remove_file(&install_data.install_path)?;
    println!("\rSuccessfully remove the {} binary.", tag_name);

    println!("Removing the {} desktop entry...", tag_name);
    fs::remove_file(&install_data.desktop_entry_path)?;
    println!("\rSuccessfully remove the {} desktop entry.", tag_name);

    println!("Updating the desktop database...");
    update_desktop_database(local_data_dir)?;
    println!("\rSuccessfully updated the desktop database!");

    Ok(())
}

#[derive(Debug)]
/// Contains common paths and file names required to manipulate a single binary.
struct InstallData {
    pub file_name: String,
    pub desktop_entry_path: PathBuf,
    pub install_path: PathBuf,
}

impl InstallData {
    fn new(local_data_dir: &Path, install_dir: &Path, release_tag_name: &str) -> Self {
        let desktop_dir = local_data_dir.join("applications");
        let app_image_file_name = format!("{}.AppImage", release_tag_name);
        let desktop_file_name = format!("osu!-{}.desktop", release_tag_name);

        Self {
            install_path: install_dir.join(&app_image_file_name),
            desktop_entry_path: desktop_dir.join(desktop_file_name),
            file_name: app_image_file_name,
        }
    }

    fn get_temp_file_path(&self) -> PathBuf {
        let temp_dir = Path::new(TEMP_DIR);
        temp_dir.join(&self.file_name)
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

        assert_eq!(install_data.file_name, String::from("2023.617.0.AppImage"));
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
