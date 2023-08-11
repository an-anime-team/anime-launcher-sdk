use std::path::PathBuf;

use md5::{Md5, Digest};

use anime_game_core::installer::downloader::Downloader;

use super::config::schema::prelude::FpsUnlockerConfig;

pub mod config_schema;

const UNLOCKER_NAME: &str = "fps_unlocker.sh";

const LATEST_INFO: (&str, &str) = (
    "aebe0a00bdb560dd5518fe945977d51c",
    "https://gist.githubusercontent.com/averageFOSSenjoyer/72af30a547088c9169628cafb075ee35/raw/0a2cf8f132420b4747364bb7b2367e339d1b7993/fps_unlocker.sh"
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FpsUnlocker {
    dir: PathBuf
}

impl FpsUnlocker {
    pub fn from_dir<T: Into<PathBuf> + std::fmt::Debug>(dir: T) -> anyhow::Result<Option<Self>> {
        let dir = dir.into();

        let hash = format!("{:x}", Md5::digest(std::fs::read(dir.join(UNLOCKER_NAME))?));

        Ok(if hash == LATEST_INFO.0 {
            Some(Self { dir })
        } else {
            None
        })
    }

    #[tracing::instrument(level = "debug")]
    pub fn download<T: Into<PathBuf> + std::fmt::Debug>(dir: T) -> anyhow::Result<Self> {
        tracing::debug!("Downloading FPS unlocker");

        let mut downloader = Downloader::new(LATEST_INFO.1)?;

        let dir = dir.into();

        // Create FPS unlocker folder if needed
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        match downloader.download(dir.join(UNLOCKER_NAME), |_, _| {}) {
            Ok(_) => Ok(Self { dir }),
            Err(err) => {
                tracing::error!("Downloading failed: {err}");

                Err(err.into())
            }
        }
    }

    #[inline]
    pub fn get_script_in<T: Into<PathBuf>>(dir: T) -> PathBuf {
        dir.into().join(UNLOCKER_NAME)
    }

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