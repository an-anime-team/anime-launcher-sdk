use std::path::PathBuf;

pub const FOLDER_NAME: &str = "wavey-launcher";

lazy_static::lazy_static! {
    // Limit max amount of log data in a file
    // This is needed to stop wine from flushing
    // tons of debug info there
    pub static ref GAME_LOG_FILE_LIMIT: usize = std::env::var("LAUNCHER_GAME_LOG_FILE_LIMIT")
        .ok()
        .and_then(|limit| limit.parse::<usize>().ok())
        .unwrap_or(8 * 1024 * 1024); // 8 MiB
}

/// Get default launcher dir path
/// 
/// If `LAUNCHER_FOLDER` variable is set, then its value will be returned. Otherwise return `$HOME/.local/share/wavey-launcher`
pub fn launcher_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("LAUNCHER_FOLDER") {
        return Ok(folder.into());
    }

    Ok(std::env::var("XDG_DATA_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.local/share"))
        .map(|home| PathBuf::from(home).join(FOLDER_NAME).canonicalize())??)
}

/// Get launcher's cache dir path
/// 
/// If `CACHE_FOLDER` variable is set, then its value will be returned. Otherwise return `$HOME/.cache/wavey-launcher`
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("CACHE_FOLDER") {
        return Ok(folder.into());
    }

    Ok(std::env::var("XDG_CACHE_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.cache"))
        .map(|home| PathBuf::from(home).join(FOLDER_NAME).canonicalize())??)
}

/// Get config file path
/// 
/// Default is `$HOME/.local/share/wavey-launcher/config.json`
pub fn config_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
