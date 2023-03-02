use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod fsr;
pub mod hud;
pub mod gamescope;

#[cfg(feature = "fps-unlocker")]
pub mod fps_unlocker;

pub mod prelude {
    pub use super::gamescope::prelude::*;

    #[cfg(feature = "fps-unlocker")]
    pub use super::fps_unlocker::prelude::*;

    pub use super::Enhancements;
    pub use super::fsr::Fsr;
    pub use super::hud::HUD;

    #[cfg(feature = "fps-unlocker")]
    pub use super::fps_unlocker::FpsUnlocker;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Enhancements {
    pub fsr: Fsr,
    pub gamemode: bool,
    pub hud: HUD,

    #[cfg(feature = "fps-unlocker")]
    pub fps_unlocker: FpsUnlocker,

    pub gamescope: Gamescope
}

impl From<&JsonValue> for Enhancements {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            fsr: match value.get("fsr") {
                Some(value) => Fsr::from(value),
                None => default.fsr
            },

            gamemode: match value.get("gamemode") {
                Some(value) => value.as_bool().unwrap_or(default.gamemode),
                None => default.gamemode
            },

            hud: match value.get("hud") {
                Some(value) => HUD::from(value),
                None => default.hud
            },

            #[cfg(feature = "fps-unlocker")]
            fps_unlocker: match value.get("fps_unlocker") {
                Some(value) => FpsUnlocker::from(value),
                None => default.fps_unlocker
            },

            gamescope: match value.get("gamescope") {
                Some(value) => Gamescope::from(value),
                None => default.gamescope
            }
        }
    }
}
