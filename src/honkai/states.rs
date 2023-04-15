use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use wincompatlib::prelude::*;

use anime_game_core::prelude::*;
use anime_game_core::honkai::prelude::*;

use crate::config::ConfigExt;
use crate::components::wine::WincompatlibWine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LauncherState {
    Launch,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable(VersionDiff),

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::Outdated`
    GameOutdated(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateUpdating {
    Game,
    Patch
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub wine_prefix: PathBuf,
    pub game_path: PathBuf,

    pub patch_servers: Vec<String>,
    pub patch_folder: PathBuf,

    pub status_updater: F
}

impl LauncherState {
    pub fn get<F: Fn(StateUpdating)>(params: LauncherStateParams<F>) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        // Check prefix existence
        if !params.wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check game installation status
        (params.status_updater)(StateUpdating::Game);

        let game = Game::new(&params.game_path);

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest(_) | VersionDiff::Predownload { .. } => {
                // Check game patch status
                /*(params.status_updater)(StateUpdating::Patch);

                let patch = Patch::new(&params.patch_folder);

                // Sync local patch folder with remote if needed
                // TODO: maybe I shouldn't do it here?
                if patch.is_sync(&params.patch_servers)?.is_none() {
                    for server in &params.patch_servers {
                        if patch.sync(server).is_ok() {
                            break;
                        }
                    }
                }

                // Check UnityPlayer patch
                let player_patch = patch.unity_player_patch()?;

                if !player_patch.is_applied(&params.game_path)? {
                    return Ok(Self::UnityPlayerPatchAvailable(player_patch));
                }

                // Check xlua patch
                if params.use_xlua_patch {
                    let xlua_patch = patch.xlua_patch()?;

                    if !xlua_patch.is_applied(&params.game_path)? {
                        return Ok(Self::XluaPatchAvailable(xlua_patch));
                    }
                }*/

                // Check if update predownload available
                if let VersionDiff::Predownload { .. } = diff {
                    Ok(Self::PredownloadAvailable(diff))
                }

                // Otherwise we can launch the game
                else {
                    Ok(Self::Launch)
                }
            }

            VersionDiff::Diff { .. } => Ok(Self::GameUpdateAvailable(diff)),
            VersionDiff::Outdated { .. } => Ok(Self::GameOutdated(diff)),
            VersionDiff::NotInstalled { .. } => Ok(Self::GameNotInstalled(diff))
        }
    }

    #[cfg(feature = "config")]
    #[tracing::instrument(level = "debug", skip(status_updater), ret)]
    pub fn get_from_config<T: Fn(StateUpdating)>(status_updater: T) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        let config = crate::honkai::config::Config::get()?;

        let mut wine_prefix = config.game.wine.prefix.clone();

        // Check wine existence
        #[cfg(feature = "components")]
        {
            if let Some(wine) = config.get_selected_wine()? {
                if !config.game.wine.builds.join(&wine.name).exists() {
                    return Ok(Self::WineNotInstalled);
                }

                let wine = wine
                    .to_wine(&config.components.path, Some(&config.game.wine.builds.join(&wine.name)))
                    .with_prefix(&config.game.wine.prefix);

                match wine {
                    WincompatlibWine::Default(wine) => if let Some(prefix) = wine.prefix {
                        wine_prefix = prefix;
                    }

                    WincompatlibWine::Proton(proton) => if let Some(prefix) = proton.wine().prefix.clone() {
                        wine_prefix = prefix;
                    }
                }
            }

            else {
                return Ok(Self::WineNotInstalled);
            }
        }

        Self::get(LauncherStateParams {
            wine_prefix,
            game_path: config.game.path,

            patch_servers: config.patch.servers,
            patch_folder: config.patch.path,

            status_updater
        })
    }
}
