use std::path::Path;
use std::{fs, io};

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
