use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::genshin::consts::launcher_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub path: PathBuf,
    pub servers: Vec<String>,
    pub apply_main: bool,
    pub apply_xlua: bool,
    pub root: bool
}

impl Default for Patch {
    #[inline]
    fn default() -> Self {
        let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

        Self {
            path: launcher_dir.join("patch"),

            servers: vec![
                String::from("https://codeberg.org/an-anime-team/dawn"),
                String::from("https://notabug.org/Krock/dawn")
            ],

            apply_main: true,
            apply_xlua: false,

            // Disable root requirement for patching if we're running launcher in flatpak
            root: !PathBuf::from("/.flatpak-info").exists()
        }
    }
}

impl From<&JsonValue> for Patch {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            path: match value.get("path") {
                Some(value) => match value.as_str() {
                    Some(value) => PathBuf::from(value),
                    None => default.path
                },
                None => default.path
            },

            servers: match value.get("servers") {
                Some(value) => match value.as_array() {
                    Some(values) => {
                        let mut servers = Vec::new();

                        for value in values {
                            if let Some(server) = value.as_str() {
                                servers.push(server.to_string());
                            }
                        }

                        servers
                    },
                    None => default.servers
                },
                None => default.servers
            },

            apply_main: match value.get("apply_main") {
                Some(value) => value.as_bool().unwrap_or(default.apply_main),
                None => default.apply_main
            },

            apply_xlua: match value.get("apply_xlua") {
                Some(value) => value.as_bool().unwrap_or(default.apply_xlua),
                None => default.apply_xlua
            },

            root: match value.get("root") {
                Some(value) => value.as_bool().unwrap_or(default.root),
                None => default.root
            }
        }
    }
}
