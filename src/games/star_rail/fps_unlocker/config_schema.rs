use serde::{Serialize, Deserialize};

use super::FpsUnlockerConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ConfigSchema {
    pub FPSTarget: u64
}

impl Default for ConfigSchema {
    fn default() -> Self {
        Self {
            FPSTarget: 60
        }
    }
}

impl ConfigSchema {
    pub fn from_config(config: FpsUnlockerConfig) -> Self {
        Self {
            FPSTarget: config.fps,
            ..Self::default()
        }
    }

    pub fn json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
