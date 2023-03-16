use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::genshin::prelude::*;

use wincompatlib::prelude::*;

use serde::{Serialize, Deserialize};

use crate::consts;
use super::components::wine::WincompatlibWine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LauncherState {
    Launch,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable {
        game: VersionDiff,
        voices: Vec<VersionDiff>
    },

    PatchAvailable(Patch),

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

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

impl Default for LauncherState {
    fn default() -> Self {
        Self::Launch
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateUpdating {
    Game,
    Voice(VoiceLocale),
    Patch
}

impl LauncherState {
    #[tracing::instrument(level = "debug", skip(status), ret)]
    pub fn get<T, F, S>(wine_prefix: T, game_path: T, voices: Vec<VoiceLocale>, patch_servers: Vec<S>, status: F) -> anyhow::Result<Self>
    where
        T: Into<PathBuf> + std::fmt::Debug,
        F: Fn(StateUpdating),
        S: ToString + std::fmt::Debug
    {
        tracing::debug!("Trying to get launcher state");

        let wine_prefix = wine_prefix.into();
        let game_path = game_path.into();

        // Check prefix existence
        if !wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check game installation status
        status(StateUpdating::Game);

        let game = Game::new(&game_path);
        let diff = game.try_get_diff()?;

        Ok(match diff {
            VersionDiff::Latest(_) | VersionDiff::Predownload { .. } => {
                let mut predownload_voice = Vec::new();

                for locale in voices {
                    let mut voice_package = VoicePackage::with_locale(locale)?;

                    status(StateUpdating::Voice(voice_package.locale()));

                    // Replace voice package struct with the one constructed in the game's folder
                    // so it'll properly calculate its difference instead of saying "not installed"
                    if voice_package.is_installed_in(&game_path) {
                        voice_package = match VoicePackage::new(get_voice_package_path(&game_path, voice_package.locale())) {
                            Some(locale) => locale,
                            None => return Err(anyhow::anyhow!("Failed to load {} voice package", voice_package.locale().to_name()))
                        };
                    }

                    let diff = voice_package.try_get_diff()?;

                    match diff {
                        VersionDiff::Latest(_) => (),
                        VersionDiff::Predownload { .. } => predownload_voice.push(diff),

                        VersionDiff::Diff { .. } => return Ok(Self::VoiceUpdateAvailable(diff)),
                        VersionDiff::Outdated { .. } => return Ok(Self::VoiceOutdated(diff)),
                        VersionDiff::NotInstalled { .. } => return Ok(Self::VoiceNotInstalled(diff))
                    }
                }

                status(StateUpdating::Patch);

                let patch = Patch::try_fetch(patch_servers, consts::PATCH_FETCHING_TIMEOUT)?;

                if patch.is_applied(&game_path)? {
                    if let VersionDiff::Predownload { .. } = diff {
                        Self::PredownloadAvailable {
                            game: diff,
                            voices: predownload_voice
                        }
                    }

                    else {
                        Self::Launch
                    }
                }

                else {
                    Self::PatchAvailable(patch)
                }
            }

            VersionDiff::Diff { .. } => Self::GameUpdateAvailable(diff),
            VersionDiff::Outdated { .. } => Self::GameOutdated(diff),
            VersionDiff::NotInstalled { .. } => Self::GameNotInstalled(diff)
        })
    }

    #[cfg(feature = "config")]
    #[tracing::instrument(level = "debug", skip(status), ret)]
    pub fn get_from_config<T: Fn(StateUpdating)>(status: T) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        let config = crate::config::get()?;

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

        let mut voices = Vec::with_capacity(config.game.voices.len());

        for voice in config.game.voices {
            voices.push(match VoiceLocale::from_str(&voice) {
                Some(locale) => locale,
                None => return Err(anyhow::anyhow!("Incorrect voice locale \"{}\" specified in the config", voice))
            });
        }

        Self::get(wine_prefix, config.game.path, voices, config.patch.servers, status)
    }
}
