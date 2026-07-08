use std::path::PathBuf;

pub const FOLDER_NAME: &str = "honkers-launcher";

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
/// If `LAUNCHER_FOLDER` variable is set, then its value will be returned.
/// Otherwise return `$HOME/.local/share/honkers-launcher`
pub fn launcher_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("LAUNCHER_FOLDER") {
        return Ok(folder.into());
    }

    let path = std::env::var("XDG_DATA_HOME")
        .map(|data| format!("{data}/{FOLDER_NAME}"))
        .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.local/share/{FOLDER_NAME}")))
        .or_else(|_| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .map(|username| format!("/home/{username}/.local/share/{FOLDER_NAME}"))
        })
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir().map(|current| current.join("data")))
        .map_err(|err| anyhow::anyhow!("Failed to find launcher folder: {err}"))?;

    path.canonicalize().or(Ok(path))
}

/// Get launcher's cache dir path
///
/// If `CACHE_FOLDER` variable is set, then its value will be returned.
/// Otherwise return `$HOME/.cache/honkers-launcher`
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("CACHE_FOLDER") {
        return Ok(folder.into());
    }

    let path = std::env::var("XDG_CACHE_HOME")
        .map(|cache| format!("{cache}/{FOLDER_NAME}"))
        .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.cache/{FOLDER_NAME}")))
        .or_else(|_| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .map(|username| format!("/home/{username}/.cache/{FOLDER_NAME}"))
        })
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir().map(|current| current.join("cache")))
        .map_err(|err| anyhow::anyhow!("Failed to find cache folder: {err}"))?;

    path.canonicalize().or(Ok(path))
}

/// Get config file path
///
/// Default is `$HOME/.local/share/honkers-launcher/config.json`
pub fn config_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
