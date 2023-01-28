use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    None,
    Popup,
    Fullscreen
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::None
    }
}

impl From<&JsonValue> for WindowMode {
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
