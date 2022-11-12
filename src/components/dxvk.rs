use std::process::Output;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use wincompatlib::prelude::*;

lazy_static::lazy_static! {
    static ref GROUPS: Vec<Group> = vec![
        Group {
            name: String::from("Vanilla"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/dxvk/vanilla.json")).unwrap().into_iter().take(12).collect()
        },
        Group {
            name: String::from("Async"),
            versions: serde_json::from_str::<Vec<Version>>(include_str!("../../components/dxvk/async.json")).unwrap().into_iter().take(12).collect()
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
    pub version: String,
    pub uri: String,
    pub recommended: bool
}

impl Version {
    /// Get latest recommended dxvk version
    pub fn latest() -> Self {
        get_groups()[0].versions[0].clone()
    }

    /// Check is current dxvk downloaded in specified folder
    pub fn is_downloaded_in<T: Into<PathBuf>>(&self, folder: T) -> bool {
        folder.into().join(&self.name).exists()
    }

    /// Apply current dxvk to specified wine prefix
    /// 
    /// If `wine_info` is `None`, then default system binaries will tried to be used
    pub fn apply<T: Into<PathBuf>>(&self, dxvks_folder: T, prefix_path: T, wine: Option<Wine>) -> anyhow::Result<Output> {
        let apply_path = dxvks_folder.into().join(&self.name).join("setup_dxvk.sh");

        let (wine_path, wineserver_path, wineboot_path) = match wine {
            Some(wine) => (wine.binary(), wine.wineserver(), wine.wineboot()),
            None => (PathBuf::from("wine64"), PathBuf::from("wineserver"), PathBuf::from("wineboot"))
        };

        let result = Dxvk::install(
            apply_path,
            prefix_path.into(),
            wine_path.clone(),
            wine_path,
            wineboot_path,
            wineserver_path
        );

        match result {
            Ok(output) => Ok(output),
            Err(err) => Err(err.into())
        }
    }
}

/// Get dxvk groups
pub fn get_groups() -> Vec<Group> {
    GROUPS.clone()
}

/// List downloaded dxvk versions in some specific folder
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

    Ok(downloaded)
}
