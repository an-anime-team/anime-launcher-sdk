use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::star_rail::consts::launcher_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub path: PathBuf
}

impl Default for Patch {
    #[inline]
    fn default() -> Self {
        let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

        Self {
            path: launcher_dir.join("patch")
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
            }
        }
    }
}
