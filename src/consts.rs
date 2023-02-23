use std::time::Duration;
use std::path::PathBuf;

/// Timeout used by `anime_game_core::telemetry::is_disabled` to check acessibility of telemetry servers
pub const TELEMETRY_CHECK_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

/// Timeout used by `anime_game_core::linux_patch::Patch::try_fetch` to fetch patch info
pub const PATCH_FETCHING_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));

/// Get default launcher dir path
/// 
/// `$HOME/.local/share/anime-game-launcher`
#[inline]
pub fn launcher_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("anime-game-launcher"))
}

/// Get default config file path
/// 
/// `$HOME/.local/share/anime-game-launcher/config.json`
#[inline]
pub fn config_file() -> Option<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
