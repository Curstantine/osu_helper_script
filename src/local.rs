use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::constants::{TEMP_DIR, USER_AGENT};
use crate::github::{self, GithubRelease, GithubReleaseAsset};
use crate::ureq::{box_request, download_file_with_progress};

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
pub fn download_release_asset(asset: &GithubReleaseAsset) -> Result<Vec<u8>, crate::errors::Error> {
    let response = box_request(
        ureq::get(&asset.browser_download_url)
            .set("Accept", "application/octet-stream")
            .set("User-Agent", USER_AGENT),
    )?;

    let server_size = response
        .header("content-length")
        .and_then(|size| size.parse::<u64>().ok())
        .unwrap_or(0);

    if server_size != asset.size {
        eprintln!(
            "The file size of the downloadable file doesn't match the size of the asset on GitHub."
        );
        std::process::exit(1);
    }

    Ok(download_file_with_progress(
        response.into_reader(),
        server_size,
    )?)
}

/// Initializes all prerequisites required to move the [download_buffer] into a
/// file and creates the desktop entry.
///
/// NOTE: This function internally handles all the errors and events, so
/// there's no need to handle them externally.
pub fn initialize_binary(
    local_data_dir: &Path,
    install_dir: &Path,
    release_tag_name: &str,
    download_buffer: Vec<u8>,
) {
    let desktop_dir = local_data_dir.join("applications");

    let app_image_file_name = format!("{}.AppImage", release_tag_name);
    let desktop_file_name = format!("osu!-{}.desktop", release_tag_name);
    let tmp_file_path = PathBuf::from(format!("{}/{}", TEMP_DIR, &app_image_file_name));

    let source_desktop_path = desktop_dir.join(desktop_file_name);
    let source_file_path = install_dir.join(&app_image_file_name);
    let source_icon_path = install_dir.join("osu.png");

    match fs::create_dir_all(TEMP_DIR) {
        Ok(_) => {
            if let Err(e) = fs::write(&tmp_file_path, download_buffer) {
                panic!(
                    "Couldn't write the downloaded file to the temporary directory:\n{:#?}",
                    e
                );
            }
        }
        Err(e) => {
            panic!("Couldn't create temporary directory at:\n{:#?}", e);
        }
    }

    if !install_dir.exists() {
        if let Err(e) = fs::create_dir_all(install_dir) {
            panic!(
                "Couldn't create the install directory at: {:#?}\n{:#?}",
                install_dir.display(),
                e
            );
        }
    }

    match fs::rename(tmp_file_path, &source_file_path) {
        Ok(_) => {
            println!(
                "Successfully installed {} to {}",
                release_tag_name,
                install_dir.display()
            );
        }
        Err(e) => {
            panic!(
                "Couldn't move the downloaded file to the install directory:\n{:#?}",
                e
            );
        }
    }

    if let Err(e) = fs::set_permissions(&source_file_path, Permissions::from_mode(0o755)) {
        panic!(
            "Couldn't set executable permissions to the downloaded file: {:#?}",
            e
        );
    }

    if !source_icon_path.exists() {
        match github::get_icon() {
            Ok(icon) => {
                if let Err(e) = fs::write(&source_icon_path, icon) {
                    eprintln!(
                        "Couldn't write the icon to the specified directory: {:#?}",
                        e
                    );
                }
            }
            Err(e) => {
                panic!("Couldn't download the icon:\n{:#?}", e);
            }
        };
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
        version = release_tag_name,
        icon_dir = source_icon_path.canonicalize().unwrap().to_str().unwrap(),
        exec_dir = source_file_path.canonicalize().unwrap().to_str().unwrap(),
    );

    match fs::write(&source_desktop_path, desktop_entry_content) {
        Ok(_) => {
            println!(
                "Successfully created the desktop entry at {}!",
                source_desktop_path.to_str().unwrap()
            );
        }
        Err(e) => {
            panic!("Couldn't create the desktop entry:\n{:#?}", e);
        }
    }

    println!("Cleaning up temporary files...");
    fs::remove_dir_all(TEMP_DIR).unwrap();
}

#[cfg(test)]
mod test {
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
}
