use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use wincompatlib::prelude::*;

lazy_static::lazy_static! {
    static ref GROUPS: Vec<Group> = vec![
        Group {
            name: String::from("Wine-GE-Proton"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/wine/wine-ge-proton.json")).unwrap().into_iter().take(12).collect()
        },
        Group {
            name: String::from("GE-Proton"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/wine/ge-proton.json")).unwrap().into_iter().take(12).collect()
        },
        Group {
            name: String::from("Soda"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/wine/soda.json")).unwrap().into_iter().take(12).collect()
        },
        Group {
            name: String::from("Lutris"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/wine/lutris.json")).unwrap().into_iter().take(12).collect()
        }
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub versions: Vec<Version>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub title: String,
    pub uri: String,
    pub files: Files,
    pub recommended: bool
}

impl Version {
    /// Get latest recommended wine version
    pub fn latest() -> Self {
        get_groups()[0].versions[0].clone()
    }

    /// Check is current wine downloaded in specified folder
    pub fn is_downloaded_in<T: Into<PathBuf>>(&self, folder: T) -> bool {
        folder.into().join(&self.name).exists()
    }

    /// Convert current wine struct to one from `wincompatlib`
    pub fn to_wine(&self) -> Wine {
        Wine::new(
            &self.files.wine64,
            None,
            Some(WineArch::Win64),
            Some(&self.files.wineboot),
            Some(&self.files.wineserver),
            WineLoader::Current
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Files {
    pub wine: String,
    pub wine64: String,
    pub wineserver: String,
    pub wineboot: String,
    pub winecfg: String
}

/// Get wine groups
pub fn get_groups() -> Vec<Group> {
    GROUPS.clone()
}

/// List downloaded wine versions in some specific folder
pub fn get_downloaded<T: Into<PathBuf>>(folder: T) -> std::io::Result<Vec<Version>> {
    let mut downloaded = Vec::new();

    let list = get_groups()
        .into_iter()
        .flat_map(|group| group.versions)
        .collect::<Vec<Version>>();

    for entry in folder.into().read_dir()? {
        let name = entry?.file_name();

        for version in &list {
            if name == version.name.as_str() {
                downloaded.push(version.clone());

                break;
            }
        }
    }

    downloaded.sort_by(|a, b| b.name.partial_cmp(&a.name).unwrap());

    Ok(downloaded)
}
