use std::path::PathBuf;

#[cfg(feature = "config")]
pub mod config;

/// Get default launcher dir path
/// 
/// `$HOME/.local/share/anime-game-launcher`
pub fn launcher_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("anime-game-launcher"))
}

/// Get default config file path
/// 
/// `$HOME/.local/share/anime-game-launcher/config.json`
pub fn config_file() -> Option<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
