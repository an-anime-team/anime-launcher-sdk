use std::path::{Path, PathBuf};

use crate::anime_game_core::traits::git_sync::RemoteGitSyncExt;

use super::wine;
use super::dxvk;

/// Try to get wine versions from components index
#[tracing::instrument(level = "debug", ret)]
#[cached::proc_macro::cached(key = "PathBuf", convert = r##"{ index.to_path_buf() }"##, result)]
pub fn get_wine_versions(index: &Path) -> anyhow::Result<Vec<wine::Group>> {
    tracing::debug!("Getting wine versions");

    let components = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(index.join("components.json"))?)?;

    match components.get("wine") {
        Some(wine) => match wine.as_array() {
            Some(groups) => {
                let mut wine_groups = Vec::with_capacity(groups.len());

                for group in groups {
                    let name = match group.get("name") {
                        Some(name) => match name.as_str() {
                            Some(name) => name.to_string(),
                            None => anyhow::bail!("Wrong components index structure: wine group's name entry must be a string")
                        }

                        None => anyhow::bail!("Wrong components index structure: wine group's name not found")
                    };

                    let title = match group.get("title") {
                        Some(title) => match title.as_str() {
                            Some(title) => title.to_string(),
                            None => anyhow::bail!("Wrong components index structure: wine group's title entry must be a string")
                        }

                        None => anyhow::bail!("Wrong components index structure: wine group's title not found")
                    };

                    let versions = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(index.join("wine").join(format!("{name}.json")))?)?;

                    let mut wine_versions = Vec::new();

                    match versions.as_array() {
                        Some(versions) => {
                            for version in versions {
                                wine_versions.push(wine::Version {
                                    name: version["name"].as_str().unwrap().to_string(),
                                    title: version["title"].as_str().unwrap().to_string(),
                                    uri: version["uri"].as_str().unwrap().to_string(),
                                    files: serde_json::from_value::<wine::Files>(version["files"].to_owned())?,
                                    features: version.get("features").map(|v| v.into())
                                });
                            }
                        }

                        None => anyhow::bail!("Wrong components index structure: wine versions must be a list")
                    }

                    wine_groups.push(wine::Group {
                        name,
                        title,
                        features: group.get("features").map(|v| v.into()),
                        versions: wine_versions
                    });
                }

                Ok(wine_groups)
            }

            None => anyhow::bail!("Wrong components index structure: wine entry must be a list")
        }

        None => anyhow::bail!("Wrong components index structure: wine entry not found")
    }
}

/// Try to get dxvk versions from components index
#[tracing::instrument(level = "debug", ret)]
#[cached::proc_macro::cached(key = "PathBuf", convert = r##"{ index.to_path_buf() }"##, result)]
pub fn get_dxvk_versions(index: &Path) -> anyhow::Result<Vec<dxvk::Group>> {
    tracing::debug!("Getting dxvk versions");

    let components = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(index.join("components.json"))?)?;

    match components.get("dxvk") {
        Some(dxvk) => match dxvk.as_array() {
            Some(groups) => {
                let mut dxvk_groups = Vec::with_capacity(groups.len());

                for group in groups {
                    let name = match group.get("name") {
                        Some(name) => match name.as_str() {
                            Some(name) => name.to_string(),
                            None => anyhow::bail!("Wrong components index structure: dxvk group's name entry must be a string")
                        }

                        None => anyhow::bail!("Wrong components index structure: dxvk group's name not found")
                    };

                    let title = match group.get("title") {
                        Some(title) => match title.as_str() {
                            Some(title) => title.to_string(),
                            None => anyhow::bail!("Wrong components index structure: dxvk group's title entry must be a string")
                        }

                        None => anyhow::bail!("Wrong components index structure: dxvk group's title not found")
                    };

                    let versions = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(index.join("dxvk").join(format!("{name}.json")))?)?;

                    let mut dxvk_versions = Vec::new();

                    match versions.as_array() {
                        Some(versions) => {
                            for version in versions {
                                dxvk_versions.push(dxvk::Version {
                                    name: version["name"].as_str().unwrap().to_string(),
                                    title: version["title"].as_str().unwrap().to_string(),
                                    version: version["version"].as_str().unwrap().to_string(),
                                    uri: version["uri"].as_str().unwrap().to_string(),
                                    features: version.get("features").map(|v| v.into())
                                });
                            }
                        }

                        None => anyhow::bail!("Wrong components index structure: wine versions must be a list")
                    }

                    dxvk_groups.push(dxvk::Group {
                        name,
                        title,
                        features: group.get("features").map(|v| v.into()),
                        versions: dxvk_versions
                    });
                }

                Ok(dxvk_groups)
            }

            None => anyhow::bail!("Wrong components index structure: wine entry must be a list")
        }

        None => anyhow::bail!("Wrong components index structure: wine entry not found")
    }
}

#[derive(Debug)]
pub struct ComponentsLoader {
    folder: PathBuf
}

impl RemoteGitSyncExt for ComponentsLoader {
    #[inline]
    fn folder(&self) -> &Path {
        self.folder.as_path()
    }
}

impl ComponentsLoader {
    #[inline]
    pub fn new<T: Into<PathBuf>>(folder: T) -> Self {
        Self {
            folder: folder.into()
        }
    }

    #[inline]
    #[tracing::instrument(level = "debug", ret)]
    /// Try to get wine versions from components index
    pub fn get_wine_versions(&self) -> anyhow::Result<Vec<wine::Group>> {
        get_wine_versions(&self.folder)
    }

    #[inline]
    #[tracing::instrument(level = "debug", ret)]
    /// Try to get dxvk versions from components index
    pub fn get_dxvk_versions(&self) -> anyhow::Result<Vec<dxvk::Group>> {
        get_dxvk_versions(&self.folder)
    }
}
