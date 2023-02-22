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
    #[tracing::instrument(level = "trace")]
    pub fn is_downloaded_in<T: Into<PathBuf> + std::fmt::Debug>(&self, folder: T) -> bool {
        folder.into().join(&self.name).exists()
    }

    /// Convert current wine struct to one from `wincompatlib`
    /// 
    /// `wine_folder` should point to the folder with wine binaries, so e.g. `/path/to/runners/wine-proton-ge-7.11`
    pub fn to_wine<T: Into<PathBuf>>(&self, wine_folder: Option<T>) -> Wine {
        let wine_folder = wine_folder.map(|folder| folder.into()).unwrap_or_default();

        Wine::new(
            wine_folder.join(&self.files.wine64),
            None,
            Some(WineArch::Win64),
            Some(wine_folder.join(&self.files.wineboot)),
            Some(wine_folder.join(&self.files.wineserver)),
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
#[tracing::instrument(level = "trace")]
pub fn get_downloaded<T: Into<PathBuf> + std::fmt::Debug>(folder: T) -> std::io::Result<Vec<Version>> {
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
