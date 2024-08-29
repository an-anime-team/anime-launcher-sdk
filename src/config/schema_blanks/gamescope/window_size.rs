use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamescopeWindowSize {
    pub width: Option<u64>,
    pub height: Option<u64>
}

impl GamescopeWindowSize {
    #[inline]
    pub fn get_command(&self, prefix: &str) -> String {
        let mut flags = Vec::with_capacity(2);

        if let Some(width) = &self.width {
            flags.push(format!("--{prefix}-width {width}"));
        }

        if let Some(height) = &self.height {
            flags.push(format!("--{prefix}-height {height}"));
        }

        flags.join(" ")
    }
}

impl From<&JsonValue> for GamescopeWindowSize {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            width: value.get("width")
                .and_then(|value| {
                    if value.is_null() {
                        Some(None)
                    } else {
                        value.as_u64().map(Some)
                    }
                })
                .unwrap_or(default.width),

            height: value.get("height")
                .and_then(|value| {
                    if value.is_null() {
                        Some(None)
                    } else {
                        value.as_u64().map(Some)
                    }
                })
                .unwrap_or(default.height)
        }
    }
}
