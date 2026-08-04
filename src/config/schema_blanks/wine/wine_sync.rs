use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum WineSync {
    None,
    ESync,
    FSync,
    NTSync
}

impl Default for WineSync {
    #[inline]
    fn default() -> Self {
        Self::NTSync
    }
}

impl From<&JsonValue> for WineSync {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl WineSync {
    /// Get environment variables corresponding to used wine sync
    pub fn get_env_vars(&self) -> HashMap<&str, &str> {
        let key = match self {
            Self::None => return HashMap::new(),

            // NTsync is used by default by every build that supports it
            Self::NTSync => return HashMap::new(),

            Self::ESync => "WINEESYNC",
            Self::FSync => "WINEFSYNC"
        };

        HashMap::from([(key, "1")])
    }
}
