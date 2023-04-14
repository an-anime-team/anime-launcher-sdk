use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum WindowType {
    Borderless,
    Fullscreen
}

impl Default for WindowType {
    #[inline]
    fn default() -> Self {
        Self::Borderless
    }
}

impl From<&JsonValue> for WindowType {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
