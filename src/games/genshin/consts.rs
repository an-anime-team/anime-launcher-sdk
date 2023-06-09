use std::path::PathBuf;

pub const FOLDER_NAME: &str = "anime-game-launcher";

/// Get default launcher dir path
/// 
/// If `LAUNCHER_FOLDER` variable is set, then its value will be returned. Otherwise return `$HOME/.local/share/anime-game-launcher`
pub fn launcher_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("LAUNCHER_FOLDER") {
        return Ok(folder.into());
    }

    Ok(std::env::var("XDG_DATA_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.local/share"))
        .map(|home| PathBuf::from(home).join(FOLDER_NAME))?)
}

/// Get launcher's cache dir path
/// 
/// If `CACHE_FOLDER` variable is set, then its value will be returned. Otherwise return `$HOME/.cache/anime-game-launcher`
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("CACHE_FOLDER") {
        return Ok(folder.into());
    }

    Ok(std::env::var("XDG_CACHE_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.cache"))
        .map(|home| PathBuf::from(home).join(FOLDER_NAME))?)
}

/// Get config file path
/// 
/// Default is `$HOME/.local/share/anime-game-launcher/config.json`
pub fn config_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
