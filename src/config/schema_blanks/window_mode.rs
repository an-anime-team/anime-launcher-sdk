use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum WindowMode {
    None,
    Popup,
    Fullscreen
}

impl Default for WindowMode {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl From<&JsonValue> for WindowMode {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
