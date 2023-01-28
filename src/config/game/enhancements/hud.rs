use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

use std::collections::HashMap;

use crate::config::Config;

#[derive(Ordinalize, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HUD {
    None,
    DXVK,
    MangoHUD
}

impl Default for HUD {
    fn default() -> Self {
        Self::None
    }
}

impl From<&JsonValue> for HUD {
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl HUD {
    /// Get environment variables corresponding to used wine hud
    pub fn get_env_vars(&self, config: &Config) -> HashMap<&str, &str> {
        match self {
            Self::None => HashMap::new(),
            Self::DXVK => HashMap::from([
                ("DXVK_HUD", "fps,frametimes,version,gpuload")
            ]),
            Self::MangoHUD => {
                // Don't show mangohud if gamescope is enabled
                // otherwise it'll be doubled
                if config.game.enhancements.gamescope.enabled {
                    HashMap::new()
                } else {
                    HashMap::from([
                        ("MANGOHUD", "1")
                    ])
                }
            }
        }
    }
}
