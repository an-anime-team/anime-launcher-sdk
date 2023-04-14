use std::process::{Command, Stdio};

pub use anime_game_core;
pub use wincompatlib;

#[cfg(feature = "genshin")]
pub mod genshin;

#[cfg(feature = "honkai")]
pub mod honkai;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "game")]
// pub mod game;

#[cfg(feature = "discord-rpc")]
// pub mod discord_rpc;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if specified binary is available
/// 
/// ```
/// assert!(anime_launcher_sdk::is_available("bash"));
/// ```
#[allow(unused_must_use)]
#[tracing::instrument(level = "trace", ret)]
pub fn is_available(binary: &str) -> bool {
    tracing::trace!("Checking package availability");

    let Ok(mut child) = Command::new(binary).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else {
        return false;
    };

    child.kill();

    true
}
