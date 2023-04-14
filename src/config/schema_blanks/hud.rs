use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum HUD {
    None,
    DXVK,
    MangoHUD
}

impl Default for HUD {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl From<&JsonValue> for HUD {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl HUD {
    /// Get environment variables corresponding to used wine hud
    pub fn get_env_vars(&self, gamescope_enabled: bool) -> HashMap<&str, &str> {
        match self {
            Self::None => HashMap::new(),
            Self::DXVK => HashMap::from([
                ("DXVK_HUD", "fps,frametimes,version,gpuload")
            ]),
            Self::MangoHUD => {
                // Don't show mangohud if gamescope is enabled
                // otherwise it'll be doubled
                if gamescope_enabled {
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
