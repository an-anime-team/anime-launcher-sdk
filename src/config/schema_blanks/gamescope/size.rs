use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Size {
    pub width: u64,
    pub height: u64
}

impl From<&JsonValue> for Size {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            width: value.get("width")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.width),

            height: value.get("height")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.height)
        }
    }
}
