use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use wincompatlib::prelude::*;

use super::loader::ComponentsLoader;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub title: String,
    pub versions: Vec<Version>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub version: String,
    pub uri: String
}

impl Version {
    /// Get latest recommended dxvk version
    pub fn latest<T: Into<PathBuf>>(components: T) -> anyhow::Result<Self> {
        Ok(get_groups(components)?[0].versions[0].clone())
    }

    /// Check is current dxvk downloaded in specified folder
    #[inline]
    pub fn is_downloaded_in<T: Into<PathBuf>>(&self, folder: T) -> bool {
        folder.into().join(&self.name).exists()
    }

    /// Install current dxvk
    #[tracing::instrument(level = "debug", ret)]
    #[inline]
    pub fn install<T: Into<PathBuf> + std::fmt::Debug>(&self, dxvks_folder: T, wine: &Wine, params: InstallParams) -> std::io::Result<()> {
        tracing::debug!("Installing DXVK");

        Dxvk::install(
            wine,
            dxvks_folder.into().join(&self.name),
            params
        )
    }

    /// Uninstall current dxvk
    #[tracing::instrument(level = "debug", ret)]
    #[inline]
    pub fn uninstall(&self, wine: &Wine, params: InstallParams) -> std::io::Result<()> {
        tracing::debug!("Uninstalling DXVK");

        Dxvk::uninstall(
            wine,
            params
        )
    }
}

pub fn get_groups<T: Into<PathBuf>>(components: T) -> anyhow::Result<Vec<Group>> {
    ComponentsLoader::new(components).get_dxvk_versions()
}

/// List downloaded dxvk versions in some specific folder
pub fn get_downloaded<T: Into<PathBuf>>(components: T, folder: T) -> anyhow::Result<Vec<Version>> {
    let mut downloaded = Vec::new();

    let list = get_groups(components)?
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
