use std::path::PathBuf;

use wincompatlib::wine::ext::Font;

use anime_game_core::prelude::*;
use anime_game_core::pgr::prelude::*;

use crate::config::ConfigExt;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    Mfc140NotInstalled,
    FontsNotInstalled(Vec<Font>),

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateUpdating {
    Components,
    Game
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub wine_prefix: PathBuf,
    pub game_path: PathBuf,
    pub fast_verify: bool,
    pub status_updater: F
}

impl LauncherState {
    pub fn get<F: Fn(StateUpdating)>(params: LauncherStateParams<F>) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        // Check prefix existence
        if !params.wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check wine components installation status
        (params.status_updater)(StateUpdating::Components);

        if !mfc140::is_installed(&params.wine_prefix) {
            return Ok(Self::Mfc140NotInstalled);
        }

        let mut fonts = Vec::new();

        // In future, wincompatlib's Font might contain fonts that won't be actually needed
        // That's why I listed only needed fonts here
        const COREFONTS: &[Font] = &[
            Font::Andale,
            Font::Arial,
            Font::Courier,
            Font::Georgia,
            Font::Impact,
            Font::Times,
            Font::Trebuchet,
            Font::Verdana,
            Font::Webdings,

            // Who even needs it?
            Font::ComicSans
        ];

        for font in COREFONTS.iter().copied() {
            if !font.is_installed(&params.wine_prefix) {
                fonts.push(font);
            }
        }

        if !fonts.is_empty() {
            return Ok(Self::FontsNotInstalled(fonts));
        }

        // Check game installation status
        (params.status_updater)(StateUpdating::Game);

        let game = Game::new(&params.game_path, ())
            .with_fast_verify(params.fast_verify);

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest(_) => Ok(Self::Launch),
            VersionDiff::Outdated { .. } => Ok(Self::GameUpdateAvailable(diff)),
            VersionDiff::NotInstalled { .. } => Ok(Self::GameNotInstalled(diff))
        }
    }

    #[cfg(feature = "config")]
    pub fn get_from_config<T: Fn(StateUpdating)>(status_updater: T) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        let config = crate::pgr::config::Config::get()?;

        match &config.game.wine.selected {
            #[cfg(feature = "components")]
            Some(selected) if !config.game.wine.builds.join(selected).exists() => return Ok(Self::WineNotInstalled),

            None => return Ok(Self::WineNotInstalled),

            _ => ()
        }

        Self::get(LauncherStateParams {
            wine_prefix: config.get_wine_prefix_path(),
            game_path: config.game.path,
            fast_verify: config.launcher.repairer.fast,

            status_updater
        })
    }
}
