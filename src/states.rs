use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::genshin::prelude::*;

use wincompatlib::prelude::*;

use serde::{Serialize, Deserialize};

use super::components::wine::WincompatlibWine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LauncherState {
    Launch,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable {
        game: VersionDiff,
        voices: Vec<VersionDiff>
    },

    UnityPlayerPatchAvailable(UnityPlayerPatch),
    XluaPatchAvailable(XluaPatch),

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub wine_prefix: PathBuf,
    pub game_path: PathBuf,

    pub selected_voices: Vec<VoiceLocale>,

    pub patch_servers: Vec<String>,
    pub patch_folder: PathBuf,
    pub use_xlua_patch: bool,

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
                let mut predownload_voice = Vec::new();

                for locale in params.selected_voices {
                    let mut voice_package = VoicePackage::with_locale(locale)?;

                    (params.status_updater)(StateUpdating::Voice(voice_package.locale()));

                    // Replace voice package struct with the one constructed in the game's folder
                    // so it'll properly calculate its difference instead of saying "not installed"
                    if voice_package.is_installed_in(&params.game_path) {
                        voice_package = match VoicePackage::new(get_voice_package_path(&params.game_path, voice_package.locale())) {
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

                // Check game patch status
                (params.status_updater)(StateUpdating::Patch);

                let patch = Patch::new(&params.patch_folder);

                // Sync local patch folder with remote if needed
                if patch.is_sync(&params.patch_servers)?.is_none() {
                    for server in &params.patch_servers {
                        if let Ok(true) = patch.sync(server) {
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
                }

                // Check if update predownload available
                if let VersionDiff::Predownload { .. } = diff {
                    Ok(Self::PredownloadAvailable {
                        game: diff,
                        voices: predownload_voice
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

        Self::get(LauncherStateParams {
            wine_prefix,
            game_path: config.game.path,

            selected_voices: voices,

            patch_servers: config.patch.servers,
            patch_folder: config.patch.path,
            use_xlua_patch: config.patch.xlua_patch,

            status_updater
        })
    }
}
