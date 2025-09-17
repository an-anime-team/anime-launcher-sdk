use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::zzz::prelude::*;

use crate::config::ConfigExt;
use crate::zzz::config::Config;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable {
        game: VersionDiff
    },

    FolderMigrationRequired {
        from: PathBuf,
        to: PathBuf,
        cleanup_folder: Option<PathBuf>
    },

    TelemetryNotDisabled,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    DxvkNotInstalled,

    // Always contains `VersionDiff::Diff`
    VoiceUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::Outdated`
    VoiceOutdated(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    VoiceNotInstalled(VersionDiff),

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::Outdated`
    GameOutdated(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateUpdating {
    Game
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub game_path: PathBuf,
    pub game_edition: GameEdition,
    pub wine_prefix: PathBuf,

    pub status_updater: F
}

impl LauncherState {
    pub fn get<F: Fn(StateUpdating)>(params: LauncherStateParams<F>) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        // Check prefix existence
        if !params.wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check dxvk installation
        let reg_path = params.wine_prefix.join("user.reg");
        let reg_content = std::fs::read_to_string(&reg_path)?;
        let mut found_dxgi = false;

        for line in reg_content.lines() {
            if line.trim_start().starts_with("\"dxgi\"") {
                found_dxgi = true;
                if !line.contains("\"native\"") {
                    return Ok(Self::DxvkNotInstalled);
                }
            }
        }
        if !found_dxgi {
           return Ok(Self::DxvkNotInstalled);
        }

        // Check game installation status
        (params.status_updater)(StateUpdating::Game);

        let game = Game::new(&params.game_path, params.game_edition);

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest { .. } | VersionDiff::Predownload { .. } => {
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

                // Check if update predownload available
                if let VersionDiff::Predownload { .. } = diff {
                    Ok(Self::PredownloadAvailable {
                        game: diff
                    })
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
            wine_prefix: config.game.wine.prefix,

            status_updater
        })
    }
}
