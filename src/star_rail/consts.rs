use std::path::PathBuf;

/// Get default launcher dir path
/// 
/// `$HOME/.local/share/honkers-railway-launcher`
#[inline]
pub fn launcher_dir() -> anyhow::Result<PathBuf> {
    Ok(std::env::var("XDG_DATA_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| home + "/.local/share"))
        .map(|home| PathBuf::from(home).join("honkers-railway-launcher"))?)
}

/// Get default config file path
/// 
/// `$HOME/.local/share/honkers-railway-launcher/config.json`
#[inline]
pub fn config_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("config.json"))
}
