use inquire::Select;
use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

use crate::{
    constants::{TEMP_DIR, USER_AGENT},
    github,
    ureq::{box_request, download_file_with_progress},
};

pub fn install(
    local_data_dir: PathBuf,
    install_dir: PathBuf,
    version: Option<String>,
) -> anyhow::Result<()> {
    let installed_versions = fs::read_dir(&install_dir)?
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

    let response = box_request(
        ureq::get(&app_image_asset.browser_download_url)
            .set("Accept", "application/octet-stream")
            .set("User-Agent", USER_AGENT),
    )?;

    let server_size = response
        .header("content-length")
        .and_then(|size| size.parse::<u64>().ok())
        .unwrap_or(0);

    if server_size != app_image_asset.size {
        eprintln!(
            "The file size of the downloadable file doesn't match the size of the asset on GitHub."
        );
        std::process::exit(1);
    }

    let file_buffer = download_file_with_progress(response.into_reader(), server_size)?;

    let app_image_file_name = format!("{}.AppImage", &version.tag_name);
    let desktop_file_name = format!("osu!-{}.desktop", &version.tag_name);

    let desktop_dir = local_data_dir.join("applications");
    let source_desktop_path = desktop_dir.join(desktop_file_name);

    let tmp_file_path = PathBuf::from(format!("{}/{}", TEMP_DIR, &app_image_file_name));
    let source_file_path = install_dir.join(&app_image_file_name);
    let source_icon_path = install_dir.join("osu.png");

    match fs::create_dir_all(TEMP_DIR) {
        Ok(_) => {
            if let Err(e) = fs::write(&tmp_file_path, file_buffer) {
                eprintln!(
                    "Couldn't write the downloaded file to the temporary directory: {:#?}",
                    e
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Couldn't create temporary directory at: {:#?}", e);
            std::process::exit(1);
        }
    }

    if !install_dir.exists() {
        if let Err(e) = fs::create_dir_all(&install_dir) {
            eprintln!(
                "Couldn't create the install directory at: {:#?}\n{:#?}",
                &install_dir.display(),
                e
            );
            std::process::exit(1);
        }
    }

    match fs::rename(tmp_file_path, &source_file_path) {
        Ok(_) => {
            println!(
                "Successfully installed {} to {}",
                &version.tag_name,
                &install_dir.display()
            );
        }
        Err(e) => {
            eprintln!(
                "Couldn't move the downloaded file to the install directory: {:#?}",
                e
            );
            std::process::exit(1);
        }
    }

    if let Err(e) = fs::set_permissions(&source_file_path, Permissions::from_mode(0o755)) {
        eprintln!(
            "Couldn't set executable permissions to the downloaded file: {:#?}",
            e
        );
        std::process::exit(1);
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
                eprintln!("Couldn't download the icon: {:#?}", e);
                std::process::exit(1);
            }
        };
    }

    let desktop_entry_content = format!(
        "[Desktop Entry]
    \r\rName=osu! {version}
    \r\rIcon={icon_dir}
    \r\rComment=rhythm is just a *click* away!
    \r\rExec={exec_dir}
    \r\rVersion=1.0
    \r\rType=Application
    \r\rCategories=Game;",
        version = &version.tag_name,
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
            eprintln!("Couldn't create the desktop entry: {:#?}", e);
            std::process::exit(1);
        }
    }

    println!("Cleaning up temporary files...");
    fs::remove_dir_all(TEMP_DIR)?;

    println!("Successfully installed osu! {}!", version.tag_name);

    Ok(())
}
