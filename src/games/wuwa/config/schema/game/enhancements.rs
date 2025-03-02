use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config::schema_blanks::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enhancements {
    pub fsr: Fsr,
    pub gamemode: bool,
    pub hud: HUD,
    pub gamescope: Gamescope,
    pub dx11: bool,
    pub fix_launch_dialog: bool
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

            gamescope: value.get("gamescope")
                .map(Gamescope::from)
                .unwrap_or(default.gamescope),

            dx11: value.get("dx11")
                .and_then(JsonValue::as_bool)
                .unwrap_or(true),

            fix_launch_dialog: value.get("fix_launch_dialog")
                .and_then(JsonValue::as_bool)
                .unwrap_or(true),
        }
    }
}
