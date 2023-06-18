use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::star_rail::prelude::*;

use crate::config::ConfigExt;
use crate::star_rail::config::Config;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    PatchNotVerified,
    PatchBroken,
    PatchUnsafe,

    PatchNotInstalled,
    PatchUpdateAvailable,

    TelemetryNotDisabled,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable(VersionDiff),

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
            VersionDiff::Latest { version, .. } | VersionDiff::Predownload { current: version, .. } => {
                // Check game patch status
                (params.status_updater)(StateUpdating::Patch);

                // Check jadeite patch status
                if !jadeite::is_installed(&params.patch_folder) {
                    return Ok(Self::PatchNotInstalled);
                }

                if jadeite::get_latest()?.version > jadeite::get_version(params.patch_folder)? {
                    return Ok(Self::PatchUpdateAvailable);
                }

                // Check telemetry servers
                let disabled = telemetry::is_disabled(params.game_edition)

                    // Return true if there's no domain name resolved, or false otherwise
                    .map(|result| result.is_none())

                    // And return true if there's an error happened during domain name resolving
                    // FIXME: might not be a good idea? Idk
                    .unwrap_or_else(|err| {
                        tracing::warn!("Failed to check telemetry servers: {err}. Assuming they're disabled");

                        true
                    });

                if !disabled {
                    return Ok(Self::TelemetryNotDisabled);
                }

                match jadeite::get_metadata()?.hsr.for_edition(params.game_edition).get_status(version) {
                    JadeitePatchStatusVariant::Verified => {
                        // Check if update predownload available
                        if let VersionDiff::Predownload { .. } = diff {
                            Ok(Self::PredownloadAvailable(diff))
                        }

                        // Otherwise we can launch the game
                        else {
                            Ok(Self::Launch)
                        }
                    }

                    JadeitePatchStatusVariant::Unverified => Ok(Self::PatchNotVerified),
                    JadeitePatchStatusVariant::Broken => Ok(Self::PatchBroken),
                    JadeitePatchStatusVariant::Unsafe => Ok(Self::PatchUnsafe)
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
            patch_folder: config.patch.path,

            status_updater
        })
    }
}
