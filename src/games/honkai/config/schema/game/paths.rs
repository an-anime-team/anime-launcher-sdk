use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use anime_game_core::honkai::consts::GameEdition;

use crate::honkai::consts::launcher_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paths {
    pub global: PathBuf,
    pub sea: PathBuf,
    pub china: PathBuf,
    pub taiwan: PathBuf,
    pub korea: PathBuf,
    pub japan: PathBuf
}

impl Paths {
    /// Get game path for given edition
    pub fn for_edition(&self, edition: impl Into<GameEdition>) -> &Path {
        match edition.into() {
            GameEdition::Global => self.global.as_path(),
            GameEdition::Sea    => self.sea.as_path(),
            GameEdition::China  => self.china.as_path(),
            GameEdition::Taiwan => self.taiwan.as_path(),
            GameEdition::Korea  => self.korea.as_path(),
            GameEdition::Japan  => self.japan.as_path()
        }
    }
}

impl Default for Paths {
    fn default() -> Self {
        let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

        Self {
            global: launcher_dir.join(concat!("Hon", "kai Imp", "act")),
            sea:    launcher_dir.join(concat!("Hon", "kai Imp", "act Sea")),
            china:  launcher_dir.join(concat!("Hon", "kai Imp", "act China")),
            taiwan: launcher_dir.join(concat!("Hon", "kai Imp", "act Taiwan")),
            korea:  launcher_dir.join(concat!("Hon", "kai Imp", "act Korea")),
            japan:  launcher_dir.join(concat!("Hon", "kai Imp", "act Japan")),
        }
    }
}

impl From<&JsonValue> for Paths {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        // SDK 1.8.13 and earlier
        if value.is_string() {
            let path = PathBuf::from(value.as_str().unwrap());

            Self {
                sea: path.parent()
                    .map(|value| value.join(concat!("Hon", "kai Imp", "act Sea")))
                    .unwrap_or(default.sea),

                china: path.parent()
                    .map(|value| value.join(concat!("Hon", "kai Imp", "act China")))
                    .unwrap_or(default.china),

                taiwan: path.parent()
                    .map(|value| value.join(concat!("Hon", "kai Imp", "act Taiwan")))
                    .unwrap_or(default.taiwan),

                korea: path.parent()
                    .map(|value| value.join(concat!("Hon", "kai Imp", "act Korea")))
                    .unwrap_or(default.korea),

                japan: path.parent()
                    .map(|value| value.join(concat!("Hon", "kai Imp", "act Japan")))
                    .unwrap_or(default.japan),

                global: path
            }
        }

        // SDK 1.9.0 and later
        else {
            Self {
                global: value.get("global")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.global),

                sea: value.get("sea")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.sea),

                china: value.get("china")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.china),

                taiwan: value.get("taiwan")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.taiwan),

                korea: value.get("korea")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.korea),

                japan: value.get("japan")
                    .and_then(JsonValue::as_str)
                    .map(PathBuf::from)
                    .unwrap_or(default.japan),
            }
        }
    }
}
