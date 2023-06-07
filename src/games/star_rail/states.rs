use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::star_rail::prelude::*;

use crate::config::ConfigExt;
use crate::star_rail::config::Config;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable(VersionDiff),

    MainPatchAvailable(MainPatch),

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateUpdating {
    Game,
    Patch
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub game_path: PathBuf,
    pub game_edition: GameEdition,

    pub wine_prefix: PathBuf,

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

        let game = Game::new(&params.game_path, params.game_edition);
        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest { .. } | VersionDiff::Predownload { .. } => {
                // Check game patch status
                (params.status_updater)(StateUpdating::Patch);

                let patch = Patch::new(&params.patch_folder, params.game_edition);

                // Sync local patch folder with remote if needed
                // TODO: maybe I shouldn't do it here?
                if patch.is_sync(&params.patch_servers)?.is_none() {
                    for server in &params.patch_servers {
                        if patch.sync(server).is_ok() {
                            break;
                        }
                    }
                }

                // Check the main patch
                let main_patch = patch.main_patch()?;

                if !main_patch.is_applied(&params.game_path)? {
                    return Ok(Self::MainPatchAvailable(main_patch));
                }

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

        let config = Config::get()?;

        match &config.game.wine.selected {
            #[cfg(feature = "components")]
            Some(selected) if !config.game.wine.builds.join(selected).exists() => return Ok(Self::WineNotInstalled),

            None => return Ok(Self::WineNotInstalled),

            _ => ()
        }

        Self::get(LauncherStateParams {
            game_path: config.game.path.for_edition(config.launcher.edition).to_path_buf(),
            game_edition: config.launcher.edition,

            wine_prefix: config.get_wine_prefix_path(),

            patch_servers: config.patch.servers,
            patch_folder: config.patch.path,

            status_updater
        })
    }
}
