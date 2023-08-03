use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Framerate {
    pub focused: u64,
    pub unfocused: u64
}

impl From<&JsonValue> for Framerate {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            focused: value.get("focused")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.focused),

            unfocused: value.get("unfocused")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.unfocused)
        }
    }
}
