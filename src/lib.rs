use std::process::{Command, Stdio};

pub mod consts;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "states")]
pub mod states;

#[cfg(feature = "components")]
pub mod components;

/// Check if specified binary is available
/// 
/// ```
/// assert!(anime_launcher_sdk::is_available("bash"));
/// ```
#[allow(unused_must_use)]
pub fn is_available(binary: &str) -> bool {
    match Command::new(binary).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(mut child) => {
            child.kill();

            true
        },
        Err(_) => false
    }
}
