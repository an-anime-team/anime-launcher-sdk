use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WineSync {
    None,
    ESync,
    FSync,
    Futex2
}

impl Default for WineSync {
    fn default() -> Self {
        Self::FSync
    }
}

impl From<&JsonValue> for WineSync {
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl WineSync {
    /// Get environment variables corresponding to used wine sync
    pub fn get_env_vars(&self) -> HashMap<&str, &str> {
        HashMap::from([(match self {
            Self::None => return HashMap::new(),

            Self::ESync  => "WINEESYNC",
            Self::FSync  => "WINEFSYNC",
            Self::Futex2 => "WINEFSYNC_FUTEX2"
        }, "1")])
    }
}
