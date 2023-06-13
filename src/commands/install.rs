use inquire::Select;
use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{
    constants::{TEMP_DIR, USER_AGENT},
    github,
    ureq::{box_request, download_file_with_progress},
};

pub fn install(
    home_dir: PathBuf,
    install_dir: String,
    version: Option<String>,
) -> anyhow::Result<()> {
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

            // TODO: Filter out already installed versions
            let release_tags = releases
                .iter()
                .map(|release| release.tag_name.clone())
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

    let install_dir_str = install_dir.replace('~', home_dir.to_str().unwrap());
    let desktop_dir_str = format!("{}/.local/share/applications", home_dir.to_str().unwrap());
    let tmp_file_path_str = format!("{}/{}", TEMP_DIR, &app_image_file_name);
    let source_file_path_str = format!("{}/{}", &install_dir_str, &app_image_file_name);
    let source_desktop_path_str = format!("{}/{}", &desktop_dir_str, &desktop_file_name);
    let source_icon_path_str = format!("{}/osu.png", &install_dir_str);

    let install_dir = Path::new(&install_dir_str);
    let tmp_file_path = Path::new(&tmp_file_path_str);
    let source_file_path = Path::new(&source_file_path_str);
    let source_desktop_path = Path::new(&source_desktop_path_str);
    let source_icon_path = Path::new(&source_icon_path_str);

    match fs::create_dir_all(TEMP_DIR) {
        Ok(_) => fs::write(tmp_file_path, file_buffer)?,
        Err(e) => {
            eprintln!("Couldn't create temporary directory at: {:#?}", e);
            std::process::exit(1);
        }
    }

    if !install_dir.exists() {
        match fs::create_dir_all(install_dir) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Couldn't create the install directory at: {:#?}\n{:#?}",
                    &install_dir_str, e
                );
                std::process::exit(1);
            }
        }
    }

    // TODO: MD5 checksum verification.
    // Currently, there's no way to get the MD5 checksum of the asset from GitHub.
    //
    // let local_md5 = md5::compute(file_buffer);
    // if format!("{:x}", local_md5) != server_md5 {
    //     eprintln!("The MD5 checksum of the downloaded file doesn't match the checksum of the asset on GitHub.");
    //     eprintln!("Expected: {:#?}\nResult: {:#?}", server_md5, local_md5);
    //     eprintln!("The file is downloaded to {:#?}, but cannot guarantee whether it's tempered/corrupted or not.", &tmp_file_path);
    //     std::process::exit(1);
    // }

    match fs::rename(tmp_file_path, source_file_path) {
        Ok(_) => {
            println!(
                "Successfully installed {} to {}!",
                &version.tag_name, &install_dir_str
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

    // set executable permission to source_file_path
    fs::set_permissions(source_file_path, Permissions::from_mode(0o755))?;

    if !source_icon_path.exists() {
        match github::get_icon() {
            Ok(icon) => {
                if let Err(e) = fs::write(source_icon_path, icon) {
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

    match fs::write(source_desktop_path, desktop_entry_content) {
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
