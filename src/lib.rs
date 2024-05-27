use std::process::{Command, Stdio};

pub use anime_game_core;
pub use wincompatlib;

mod games;

#[cfg(feature = "genshin")]
pub use games::genshin;

#[cfg(feature = "star-rail")]
pub use games::star_rail;

#[cfg(feature = "honkai")]
pub use games::honkai;

#[cfg(feature = "pgr")]
pub use games::pgr;

#[cfg(feature = "wuwa")]
pub use games::wuwa;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "discord-rpc")]
pub mod discord_rpc;

#[cfg(feature = "sessions")]
pub mod sessions;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: rewrite it to find this binary in PATH instead

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
