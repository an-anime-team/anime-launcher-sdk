use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config::schema_blanks::prelude::*;

#[cfg(feature = "fps-unlocker")]
use super::FpsUnlocker;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
