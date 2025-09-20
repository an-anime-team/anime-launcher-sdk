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
    PatchConcerning,

    PatchNotInstalled,
    PatchUpdateAvailable,

    TelemetryNotDisabled,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    DxvkNotInstalled,

    /// Always contains `VersionDiff::Predownload`
    PredownloadAvailable {
        game: VersionDiff,
        voices: Vec<VersionDiff>,
        patch: JadeitePatchStatusVariant
    },

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
    Game,
    Voice(VoiceLocale),
    Patch
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub game_path: PathBuf,
    pub game_edition: GameEdition,

    pub wine_prefix: PathBuf,
    pub patch_folder: PathBuf,

    pub selected_voices: Vec<VoiceLocale>,
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
            VersionDiff::Latest { version, .. } | VersionDiff::Predownload { current: version, .. } => {
                let mut predownload_voice = Vec::new();

                for locale in params.selected_voices {
                    let mut voice_package = VoicePackage::with_locale(locale, params.game_edition)?;

                    (params.status_updater)(StateUpdating::Voice(voice_package.locale()));

                    // Replace voice package struct with the one constructed in the game's folder
                    // so it'll properly calculate its difference instead of saying "not installed"
                    if voice_package.is_installed_in(&params.game_path) {
                        voice_package = match VoicePackage::new(get_voice_package_path(&params.game_path, params.game_edition, voice_package.locale()), params.game_edition) {
                            Some(locale) => locale,
                            None => return Err(anyhow::anyhow!("Failed to load {} voice package", voice_package.locale().to_name()))
                        };
                    }

                    let diff = voice_package.try_get_diff()?;

                    match diff {
                        VersionDiff::Latest { .. } => (),
                        VersionDiff::Predownload { .. } => predownload_voice.push(diff),

                        VersionDiff::Diff { .. } => return Ok(Self::VoiceUpdateAvailable(diff)),
                        VersionDiff::Outdated { .. } => return Ok(Self::VoiceOutdated(diff)),
                        VersionDiff::NotInstalled { .. } => return Ok(Self::VoiceNotInstalled(diff))
                    }
                }

                // Check game patch status
                (params.status_updater)(StateUpdating::Patch);

                // Check jadeite patch status
                if !jadeite::is_installed(&params.patch_folder) {
                    return Ok(Self::PatchNotInstalled);
                }

                // Fetch patch metadata
                let metadata = jadeite::get_metadata()?;

                if metadata.jadeite.version > jadeite::get_version(params.patch_folder)? {
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
                
                // Request current patch status from the metadata file
                let patch = metadata.games.hsr
                    .for_edition(params.game_edition)
                    .get_status(version);

                // Check if update predownload available
                if let VersionDiff::Predownload { .. } = diff {
                    Ok(Self::PredownloadAvailable {
                        game: diff,
                        voices: predownload_voice,
                        patch
                    })
                }

                // Otherwise we can launch the game or say that the patch is unstable
                else {
                    match patch {
                        JadeitePatchStatusVariant::Verified   => Ok(Self::Launch),
                        JadeitePatchStatusVariant::Unverified => Ok(Self::PatchNotVerified),
                        JadeitePatchStatusVariant::Broken     => Ok(Self::PatchBroken),
                        JadeitePatchStatusVariant::Unsafe     => Ok(Self::PatchUnsafe),
                        JadeitePatchStatusVariant::Concerning => Ok(Self::PatchConcerning)
                    }
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

        let mut voices = Vec::with_capacity(config.game.voices.len());

        for voice in &config.game.voices {
            voices.push(match VoiceLocale::from_str(voice) {
                Some(locale) => locale,
                None => return Err(anyhow::anyhow!("Incorrect voice locale \"{}\" specified in the config", voice))
            });
        }

        Self::get(LauncherStateParams {
            game_path: config.game.path.for_edition(config.launcher.edition).to_path_buf(),
            game_edition: config.launcher.edition,

            wine_prefix: config.game.wine.prefix,
            patch_folder: config.patch.path,

            selected_voices: voices,
            status_updater
        })
    }
}
