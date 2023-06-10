use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::honkai::prelude::*;

use crate::config::ConfigExt;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    MfplatPatchAvailable,
    MainPatchAvailable(MainPatch),

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateUpdating {
    Game,
    Patch
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub wine_prefix: PathBuf,
    pub game_path: PathBuf,

    pub patch_servers: Vec<String>,
    pub patch_folder: PathBuf,
    pub apply_mfplat: bool,

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

        let game = Game::new(&params.game_path, ());

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest(_) => {
                // Check game patch status
                (params.status_updater)(StateUpdating::Patch);

                // Check if mfplat patch is needed
                if params.apply_mfplat && !MfplatPatch::is_applied(&params.wine_prefix)? {
                    return Ok(Self::MfplatPatchAvailable);
                }

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

                // Check main patch status
                let player_patch = patch.main_patch()?;

                if !player_patch.is_applied(&params.game_path)? {
                    return Ok(Self::MainPatchAvailable(player_patch));
                }

                Ok(Self::Launch)
            }

            VersionDiff::Diff { .. } => Ok(Self::GameUpdateAvailable(diff)),
            VersionDiff::NotInstalled { .. } => Ok(Self::GameNotInstalled(diff))
        }
    }

    #[cfg(feature = "config")]
    #[tracing::instrument(level = "debug", skip(status_updater), ret)]
    pub fn get_from_config<T: Fn(StateUpdating)>(status_updater: T) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        let config = crate::honkai::config::Config::get()?;

        match &config.game.wine.selected {
            #[cfg(feature = "components")]
            Some(selected) if !config.game.wine.builds.join(selected).exists() => return Ok(Self::WineNotInstalled),

            None => return Ok(Self::WineNotInstalled),

            _ => ()
        }

        Self::get(LauncherStateParams {
            wine_prefix: config.get_wine_prefix_path(),
            game_path: config.game.path,

            patch_servers: config.patch.servers,
            patch_folder: config.patch.path,
            apply_mfplat: config.patch.apply_mfplat,

            status_updater
        })
    }
}
