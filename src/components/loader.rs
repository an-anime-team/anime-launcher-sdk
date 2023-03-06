use std::path::{Path, PathBuf};

use crate::anime_game_core::traits::git_sync::RemoteGitSync;
use super::wine;
use super::dxvk;

#[derive(Debug)]
pub struct ComponentsLoader {
    folder: PathBuf
}

impl RemoteGitSync for ComponentsLoader {
    fn folder(&self) -> &Path {
        self.folder.as_path()
    }
}

impl ComponentsLoader {
    pub fn new<T: Into<PathBuf>>(folder: T) -> Self {
        Self {
            folder: folder.into()
        }
    }

    /// Try to get wine versions from components index
    #[tracing::instrument(level = "debug", ret)]
    pub fn get_wine_versions(&self) -> anyhow::Result<Vec<wine::Group>> {
        tracing::debug!("Getting wine versions");

        let components = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(self.folder.join("components.json"))?)?;

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

                        let versions = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(self.folder.join("wine").join(format!("{name}.json")))?)?;

                        let mut wine_versions = Vec::new();

                        match versions.as_array() {
                            Some(versions) => {
                                for version in versions {
                                    wine_versions.push(serde_json::from_value::<wine::Version>(version.to_owned())?);
                                }
                            }

                            None => anyhow::bail!("Wrong components index structure: wine versions must be a list")
                        }

                        wine_groups.push(wine::Group {
                            name,
                            title,
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
    pub fn get_dxvk_versions(&self) -> anyhow::Result<Vec<dxvk::Group>> {
        tracing::debug!("Getting dxvk versions");

        let components = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(self.folder.join("components.json"))?)?;

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

                        let versions = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(self.folder.join("dxvk").join(format!("{name}.json")))?)?;

                        let mut dxvk_versions = Vec::new();

                        match versions.as_array() {
                            Some(versions) => {
                                for version in versions {
                                    dxvk_versions.push(serde_json::from_value::<dxvk::Version>(version.to_owned())?);
                                }
                            }

                            None => anyhow::bail!("Wrong components index structure: wine versions must be a list")
                        }

                        dxvk_groups.push(dxvk::Group {
                            name,
                            title,
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
}
