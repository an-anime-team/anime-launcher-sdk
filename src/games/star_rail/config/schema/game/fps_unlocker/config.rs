use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub fps: u64
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            fps: 60
        }
    }
}

impl From<&JsonValue> for Config {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            fps: match value.get("fps") {
                Some(value) => value.as_u64().unwrap_or(default.fps),
                None => default.fps
            }
        }
    }
}
