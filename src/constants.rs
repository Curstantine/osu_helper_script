pub const USER_AGENT: &str = concat!("osu_install_script/", env!("CARGO_PKG_VERSION"));

pub const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/ppy/osu/releases";
pub const GITHUB_RELEASE_TAG_URL: &str = "https://api.github.com/repos/ppy/osu/releases/tags";
pub const GITHUB_LATEST_RELEASE_URL: &str = "https://api.github.com/repos/ppy/osu/releases/latest";
pub const GITHUB_ICON_URL: &str =
    "https://raw.githubusercontent.com/ppy/osu/master/assets/lazer-nuget.png";

pub const TEMP_DIR: &str = "/var/tmp/osu_helper_script";
