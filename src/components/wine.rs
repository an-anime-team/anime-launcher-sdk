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
    pub title: String,
    pub uri: String,
    pub files: Files
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Files {
    pub wine: String,
    pub wine64: Option<String>,
    pub wineserver: Option<String>,
    pub wineboot: Option<String>,
    pub winecfg: Option<String>
}

impl Version {
    /// Get latest recommended wine version
    pub fn latest<T: Into<PathBuf>>(components: T) -> anyhow::Result<Self> {
        Ok(get_groups(components)?[0].versions[0].clone())
    }

    /// Check is current wine downloaded in specified folder
    #[inline]
    pub fn is_downloaded_in<T: Into<PathBuf>>(&self, folder: T) -> bool {
        folder.into().join(&self.name).exists()
    }

    /// Convert current wine struct to one from `wincompatlib`
    /// 
    /// `wine_folder` should point to the folder with wine binaries, so e.g. `/path/to/runners/wine-proton-ge-7.11`
    #[inline]
    pub fn to_wine<T: Into<PathBuf>>(&self, wine_folder: Option<T>) -> Wine {
        let wine_folder = wine_folder.map(|folder| folder.into()).unwrap_or_default();

        let (wine, arch) = match self.files.wine64.as_ref() {
            Some(wine) => (wine, WineArch::Win64),
            None => (&self.files.wine, WineArch::Win32)
        };

        let wineboot = self.files.wineboot.as_ref().map(|wineboot| wine_folder.join(wineboot));
        let wineserver = self.files.wineserver.as_ref().map(|wineserver| wine_folder.join(wineserver));

        Wine::new(
            wine_folder.join(wine),
            None,
            Some(arch),
            wineboot,
            wineserver,
            WineLoader::Current
        )
    }
}

pub fn get_groups<T: Into<PathBuf>>(components: T) -> anyhow::Result<Vec<Group>> {
    ComponentsLoader::new(components).get_wine_versions()
}

/// List downloaded wine versions in some specific folder
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

    downloaded.sort_by(|a, b| b.name.partial_cmp(&a.name).unwrap());

    Ok(downloaded)
}
