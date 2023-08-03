use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config::schema_blanks::resolution::Resolution;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualDesktop {
    pub enabled: bool,
    pub width: u64,
    pub height: u64
}

impl Default for VirtualDesktop {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: false,
            width: 1920,
            height: 1080
        }
    }
}

impl From<&JsonValue> for VirtualDesktop {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            enabled: value.get("enabled")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.enabled),

            width: value.get("width")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.width),

            height: value.get("height")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.height)
        }
    }
}

impl VirtualDesktop {
    #[inline]
    pub fn get_resolution(&self) -> Resolution {
        Resolution::from_pair(self.width, self.height)
    }

    #[inline]
    /// `explorer /desktop=[desktop_name],[width]x[height]`
    pub fn get_command<T: AsRef<str>>(&self, desktop_name: T) -> Option<String> {
        if self.enabled {
            Some(format!("explorer /desktop={},{}x{}", desktop_name.as_ref(), self.width, self.height))
        }

        else {
            None
        }
    }
}
