use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::honkai::consts::launcher_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub path: PathBuf,
    pub apply_mfplat: bool
}

impl Default for Patch {
    #[inline]
    fn default() -> Self {
        let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

        Self {
            path: launcher_dir.join("patch"),

            // Seems to not be needed with wine 8+
            // which is recommended by default, so will work
            // for most of users
            apply_mfplat: false
        }
    }
}

impl From<&JsonValue> for Patch {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            path: match value.get("path").and_then(|path| path.as_str()).map(PathBuf::from) {
                Some(path) => path,
                None => default.path
            },

            apply_mfplat: match value.get("apply_mfplat") {
                Some(value) => value.as_bool().unwrap_or(default.apply_mfplat),
                None => default.apply_mfplat
            }
        }
    }
}
