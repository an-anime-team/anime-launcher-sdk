use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repairer {
    pub threads: u64,
    pub fast: bool
}

impl Default for Repairer {
    #[inline]
    fn default() -> Self {
        Self {
            threads: 4,
            fast: false
        }
    }
}

impl From<&JsonValue> for Repairer {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            threads: value.get("threads")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.threads),

            fast: value.get("fast")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.fast)
        }
    }
}
