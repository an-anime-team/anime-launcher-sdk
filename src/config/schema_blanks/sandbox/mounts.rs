use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mounts {
    /// Bind original directory into the sandbox in read-only state
    pub read_only: HashMap<String, String>,

    /// Bind original directory into the sandbox with writing access
    pub bind: HashMap<String, String>,

    /// Symlink original files into sandbox with writing access
    pub symlinks: HashMap<String, String>
}

impl From<&JsonValue> for Mounts {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            read_only: match value.get("read_only") {
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
                    None => default.read_only
                },
                None => default.read_only
            },

            bind: match value.get("bind") {
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
                    None => default.bind
                },
                None => default.bind
            },

            symlinks: match value.get("symlinks") {
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
                    None => default.symlinks
                },
                None => default.symlinks
            }
        }
    }
}
