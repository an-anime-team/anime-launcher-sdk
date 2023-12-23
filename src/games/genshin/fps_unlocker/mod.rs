use std::path::PathBuf;

use md5::{Md5, Digest};

use anime_game_core::installer::downloader::Downloader;

use super::config::schema::prelude::FpsUnlockerConfig;

pub mod config_schema;

const LATEST_INFO: (&str, &str) = (
    "cff81830eebd3566d51b73ffaa444035",
    "https://github.com/34736384/genshin-fps-unlock/releases/download/v2.2.0/unlockfps_clr.exe"
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FpsUnlocker {
    dir: PathBuf
}

impl FpsUnlocker {
    /// Get FpsUnlocker from its containment directory
    /// 
    /// Returns
    /// - `Err(..)` if failed to read `unlocker.exe` file
    /// - `Ok(None)` if version is not latest
    /// - `Ok(..)` if version is latest
    pub fn from_dir<T: Into<PathBuf> + std::fmt::Debug>(dir: T) -> anyhow::Result<Option<Self>> {
        let dir = dir.into();

        let hash = format!("{:x}", Md5::digest(std::fs::read(dir.join("unlocker.exe"))?));

        Ok(if hash == LATEST_INFO.0 {
            Some(Self { dir })
        } else {
            None
        })
    }

    /// Download FPS unlocker to specified directory
    #[tracing::instrument(level = "debug")]
    pub fn download<T: Into<PathBuf> + std::fmt::Debug>(dir: T) -> anyhow::Result<Self> {
        tracing::debug!("Downloading FPS unlocker");

        let mut downloader = Downloader::new(LATEST_INFO.1)?;

        let dir = dir.into();

        // Create FPS unlocker folder if needed
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        match downloader.download(dir.join("unlocker.exe"), |_, _| {}) {
            Ok(_) => Ok(Self { dir }),
            Err(err) => {
                tracing::error!("Downloading failed: {err}");

                Err(err.into())
            }
        }
    }

    #[inline]
    pub fn get_binary(&self) -> PathBuf {
        Self::get_binary_in(&self.dir)
    }

    #[inline]
    pub fn get_binary_in<T: Into<PathBuf>>(dir: T) -> PathBuf {
        dir.into().join("unlocker.exe")
    }

    #[inline]
    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    /// Generate and save FPS unlocker config file to the game's directory
    #[tracing::instrument(level = "debug", ret)]
    pub fn update_config(&self, config: FpsUnlockerConfig) -> anyhow::Result<()> {
        tracing::debug!("Updating FPS unlocker config");

        let config = config_schema::ConfigSchema::from_config(config);

        Ok(std::fs::write(
            self.dir.join("fps_config.json"),
            config.json()?
        )?)
    }
}
