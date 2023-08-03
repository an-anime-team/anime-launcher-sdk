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
            fsr: value.get("fsr")
                .map(Fsr::from)
                .unwrap_or(default.fsr),

            gamemode: value.get("gamemode")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.gamemode),

            hud: value.get("hud")
                .map(HUD::from)
                .unwrap_or(default.hud),

            #[cfg(feature = "fps-unlocker")]
            fps_unlocker: value.get("fps_unlocker")
                .map(FpsUnlocker::from)
                .unwrap_or(default.fps_unlocker),

            gamescope: value.get("gamescope")
                .map(Gamescope::from)
                .unwrap_or(default.gamescope)
        }
    }
}
