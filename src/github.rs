use serde::Deserialize;

use crate::{
    constants::{
        GITHUB_ICON_URL, GITHUB_LATEST_RELEASE_URL, GITHUB_RELEASES_URL, GITHUB_RELEASE_TAG_URL,
        USER_AGENT,
    },
    errors::{self, Error},
    net,
};

pub fn get_releases() -> Result<Vec<GithubRelease>, Box<ureq::Error>> {
    net::box_and_deserialize::<Vec<GithubRelease>>(
        ureq::get(GITHUB_RELEASES_URL).set("User-Agent", USER_AGENT),
    )
}

pub fn get_release(tag: &str) -> Result<GithubRelease, Box<ureq::Error>> {
    net::box_and_deserialize::<GithubRelease>(
        ureq::get(&format!("{}/{}", GITHUB_RELEASE_TAG_URL, tag)).set("User-Agent", USER_AGENT),
    )
}

pub fn get_latest_release() -> Result<GithubRelease, Box<ureq::Error>> {
    net::box_and_deserialize::<GithubRelease>(
        ureq::get(GITHUB_LATEST_RELEASE_URL).set("User-Agent", USER_AGENT),
    )
}

pub fn get_icon() -> errors::Result<Vec<u8>> {
    let response = net::box_request(ureq::get(GITHUB_ICON_URL).set("User-Agent", USER_AGENT))?;
    let size = match response.header("Content-Length").unwrap().parse::<u64>() {
        Ok(size) => size,
        Err(_) => return Err(Error::Descriptive("Couldn't parse icon size".into())),
    };

    let mut icon = Vec::with_capacity(size as usize);
    response.into_reader().read_to_end(&mut icon)?;

    Ok(icon)
}

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub id: u64,
    pub tag_name: String,
    pub prerelease: bool,
    pub assets: Vec<GithubReleaseAsset>,
}

impl GithubRelease {
    pub fn get_app_image_asset(&self) -> Option<&GithubReleaseAsset> {
        self.assets
            .iter()
            .find(|asset| asset.name.ends_with(".AppImage"))
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}
