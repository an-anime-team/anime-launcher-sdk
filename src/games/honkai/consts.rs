use std::path::PathBuf;

/// Get default launcher dir path
/// 
/// If `LAUNCHER_FOLDER` variable is set, then its value will be returned. Otherwise return `$HOME/.local/share/honkers-launcher`
#[inline]
pub fn launcher_dir() -> anyhow::Result<PathBuf> {
    if let Ok(folder) = std::env::var("LAUNCHER_FOLDER") {
        return Ok(folder.into());
    }

    Ok(std::env::var("XDG_DATA_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.local/share"))
        .map(|home| PathBuf::from(home).join("honkers-launcher"))?)
}

/// Get default config file path
/// 
/// `$HOME/.local/share/honkers-launcher/config.json`
#[inline]
pub fn config_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
