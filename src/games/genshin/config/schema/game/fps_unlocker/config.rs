use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub fps: u64, // TODO: Fps enum
    pub interval: u64
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            fps: 120,
            interval: 5000
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
            },

            interval: match value.get("interval") {
                Some(value) => value.as_u64().unwrap_or(default.interval),
                None => default.interval
            }
        }
    }
}
