use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::wine;
use super::dxvk;

/// Try to get wine versions from components index
#[tracing::instrument(level = "debug")]
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
                                    format: version["format"].as_str().map(|str| str.to_string()),
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
#[tracing::instrument(level = "debug")]
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
                                    format: version["format"].as_str().map(|str| str.to_string()),
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

impl ComponentsLoader {
    #[inline]
    pub fn new<T: Into<PathBuf>>(folder: T) -> Self {
        Self {
            folder: folder.into()
        }
    }

    /// Verify that the local components index is synced with any of the given
    /// remotes.
    ///
    /// Returns the remote the local repository is synced with, or `None` if
    /// it's not synced with any of them.
    pub fn is_sync<T, F>(&self, remotes: T) -> anyhow::Result<Option<String>>
    where
        T: IntoIterator<Item = F>,
        F: AsRef<str>
    {
        tracing::trace!("Checking local repository sync state: {:?}", self.folder);

        if !self.folder.exists() {
            tracing::warn!("Given local repository folder doesn't exist");

            return Ok(None);
        }

        for remote in remotes {
            match self.is_sync_with(remote.as_ref()) {
                Ok(true) => return Ok(Some(remote.as_ref().to_string())),

                Ok(false) => (),

                Err(err) => {
                    tracing::warn!(
                        "Failed to check sync state with {}: {err}",
                        remote.as_ref()
                    );
                }
            }
        }

        Ok(None)
    }

    /// Verify that the local components index is synced with the given remote.
    pub fn is_sync_with(&self, remote: impl AsRef<str>) -> anyhow::Result<bool> {
        tracing::trace!(
            "Checking local repository sync state. Folder: {:?}. Remote: {}",
            self.folder,
            remote.as_ref()
        );

        if !self.folder.join(".git").exists() {
            tracing::warn!("Given local repository folder is not a git repository");

            return Ok(false);
        }

        let head = git(&self.folder, &["rev-parse", "HEAD"])?;

        git(&self.folder, &["remote", "set-url", "origin", remote.as_ref()])?;
        git(&self.folder, &["fetch", "origin"])?;

        let remote_head = git(&self.folder, &["rev-parse", "origin/HEAD"])?;

        Ok(head == remote_head)
    }

    /// Fetch the components index from the given remote.
    ///
    /// Returns the list of changes (commit messages) applied during the sync.
    pub fn sync(&self, remote: impl AsRef<str>) -> anyhow::Result<Vec<String>> {
        tracing::debug!("Syncing local components index with remote");

        if self.folder.join(".git").exists() {
            let head_commit = String::from_utf8(
                git(&self.folder, &["rev-parse", "HEAD"])?
            )?
                .trim_end()
                .to_string();

            git(&self.folder, &["remote", "set-url", "origin", remote.as_ref()])?;
            git(&self.folder, &["fetch", "origin"])?;
            git(&self.folder, &["reset", "--hard", "origin/HEAD"])?;

            let range = format!("{head_commit}..HEAD");

            let changes = String::from_utf8(
                git(&self.folder, &["--no-pager", "log", "--oneline", range.as_str()])?
            )?;

            Ok(parse_changes(&changes))
        }

        else {
            if let Some(parent) = self.folder.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let output = Command::new("git")
                .arg("clone")
                .arg(remote.as_ref())
                .arg(&self.folder)
                .env("GIT_TERMINAL_PROMPT", "0")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                anyhow::bail!(
                    "git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }

            let changes = String::from_utf8(
                git(&self.folder, &["--no-pager", "log", "--oneline"])?
            )?;

            Ok(parse_changes(&changes))
        }
    }

    #[inline]
    #[tracing::instrument(level = "debug")]
    /// Try to get wine versions from components index
    pub fn get_wine_versions(&self) -> anyhow::Result<Vec<wine::Group>> {
        get_wine_versions(&self.folder)
    }

    #[inline]
    #[tracing::instrument(level = "debug")]
    /// Try to get dxvk versions from components index
    pub fn get_dxvk_versions(&self) -> anyhow::Result<Vec<dxvk::Group>> {
        get_dxvk_versions(&self.folder)
    }
}

fn git(folder: &Path, args: &[&str]) -> anyhow::Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(folder)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(output.stdout)
}

fn parse_changes(changes: &str) -> Vec<String> {
    changes
        .trim_end()
        .lines()
        .filter_map(|line| {
            line.split_once(' ')
                .map(|(_, message)| message.to_string())
        })
        .collect()
}
