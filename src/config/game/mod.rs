use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod paths;
pub mod wine;
pub mod dxvk;
pub mod enhancements;

pub mod prelude {
    pub use super::enhancements::prelude::*;
    pub use super::wine::prelude::*;

    pub use super::paths::Paths;
    pub use super::Game;
    pub use super::dxvk::Dxvk;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub path: Paths,
    pub voices: Vec<String>,
    pub wine: Wine,
    pub dxvk: Dxvk,
    pub enhancements: prelude::Enhancements,
    pub environment: HashMap<String, String>,
    pub command: Option<String>
}

impl Default for Game {
    fn default() -> Self {
        Self {
            path: Paths::default(),
            voices: vec![
                String::from("en-us")
            ],
            wine: Wine::default(),
            dxvk: Dxvk::default(),
            enhancements: Enhancements::default(),
            environment: HashMap::new(),
            command: None
        }
    }
}

impl From<&JsonValue> for Game {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            path: match value.get("path") {
                Some(value) => Paths::from(value),
                None => default.path
            },

            voices: match value.get("voices") {
                Some(value) => match value.as_array() {
                    Some(values) => {
                        let mut voices = Vec::new();

                        for value in values {
                            if let Some(voice) = value.as_str() {
                                voices.push(voice.to_string());
                            }
                        }

                        voices
                    },
                    None => default.voices
                },
                None => default.voices
            },

            wine: match value.get("wine") {
                Some(value) => Wine::from(value),
                None => default.wine
            },

            dxvk: match value.get("dxvk") {
                Some(value) => Dxvk::from(value),
                None => default.dxvk
            },

            enhancements: match value.get("enhancements") {
                Some(value) => Enhancements::from(value),
                None => default.enhancements
            },

            environment: match value.get("environment") {
                Some(value) => match value.as_object() {
                    Some(values) => {
                        let mut vars = HashMap::new();

                        for (name, value) in values {
                            if let Some(value) = value.as_str() {
                                vars.insert(name.clone(), value.to_string());
                            }
                        }

                        vars
                    },
                    None => default.environment
                },
                None => default.environment
            },

            command: match value.get("command") {
                Some(value) => {
                    if value.is_null() {
                        None
                    } else {
                        match value.as_str() {
                            Some(value) => Some(value.to_string()),
                            None => default.command
                        }
                    }
                },
                None => default.command
            }
        }
    }
}
