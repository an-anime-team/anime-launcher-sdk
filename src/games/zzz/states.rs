use std::path::PathBuf;

use anime_game_core::zzz::prelude::*;

use crate::config::ConfigExt;
use crate::zzz::config::Config;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,
    DxvkNotInstalled,
    Dx12NotInstalled,

    FolderMigrationRequired {
        from: PathBuf,
        to: PathBuf,
        cleanup_folder: Option<PathBuf>
    },

    GameDiff(VersionDiff)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams {
    pub wine_prefix: PathBuf,
    pub enable_dx12: bool
}

impl LauncherState {
    /// Detect the current launcher state.
    pub fn get(game: &Game, params: LauncherStateParams) -> anyhow::Result<Self> {
        tracing::debug!("obtaining launcher state");

        // Check prefix existence.
        if !params.wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check dxvk installation.
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

        if params.enable_dx12 {
            // check vkd3d-proton installation
            // we only look for the d3d12 override since install_dx12
            // always installs vkd3d-proton and dxvk-nvapi together
            let mut found_d3d12 = false;

            for line in reg_content.lines() {
                if line.trim_start().starts_with("\"d3d12\"") {
                    found_d3d12 = true;

                    if !line.contains("\"native\"") {
                        return Ok(Self::Dx12NotInstalled);
                    }
                }
            }

            if !found_d3d12 {
                return Ok(Self::Dx12NotInstalled);
            }
        }

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest {
                ..
            } => Ok(Self::Launch),
            _ => Ok(Self::GameDiff(diff))
        }
    }

    #[cfg(feature = "config")]
    pub fn get_from_config() -> anyhow::Result<Self> {
        use anime_game_core::traits::game::GameExt;

        tracing::debug!("Trying to get launcher state");

        let config = Config::get()?;

        #[cfg(feature = "components")]
        match &config.game.wine.selected {
            Some(selected) if !config.game.wine.builds.join(selected).exists() => {
                return Ok(Self::WineNotInstalled);
            }

            None => return Ok(Self::WineNotInstalled),

            _ => ()
        }

        let game = Game::new(
            config
                .game
                .path
                .for_edition(&config.launcher.edition)
                .to_owned(),
            config.launcher.edition
        );

        Self::get(&game, LauncherStateParams {
            wine_prefix: config.game.wine.prefix,
            enable_dx12: config.game.enhancements.dx12
        })
    }
}
