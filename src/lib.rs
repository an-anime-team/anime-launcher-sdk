use std::process::{Command, Stdio};

pub use anime_game_core;
pub use wincompatlib;

pub mod consts;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "states")]
pub mod states;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "game")]
pub mod game;

#[cfg(feature = "fps-unlocker")]
pub mod fps_unlocker;

/// Check if specified binary is available
/// 
/// ```
/// assert!(anime_launcher_sdk::is_available("bash"));
/// ```
#[allow(unused_must_use)]
#[tracing::instrument(level = "trace", ret)]
pub fn is_available(binary: &str) -> bool {
    tracing::trace!("Checking package availability");

    match Command::new(binary).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(mut child) => {
            child.kill();

            true
        },
        Err(_) => false
    }
}
