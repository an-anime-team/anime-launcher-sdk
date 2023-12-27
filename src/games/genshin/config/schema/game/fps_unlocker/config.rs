use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub fps: u64, // TODO: Fps enum
    pub periodic_writes: bool,
    pub interval: u64
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            fps: 120,
            periodic_writes: true,
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

            periodic_writes: match value.get("periodic_writes") {
                Some(value) => value.as_bool().unwrap_or(default.periodic_writes),
                None => default.periodic_writes
            },

            interval: match value.get("interval") {
                Some(value) => value.as_u64().unwrap_or(default.interval),
                None => default.interval
            }
        }
    }
}
