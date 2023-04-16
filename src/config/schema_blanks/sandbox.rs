use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sandbox {
    pub enabled: bool,
    pub private: Vec<String>
}

impl Default for Sandbox {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: true,
            private: vec![]
        }
    }
}

impl From<&JsonValue> for Sandbox {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            enabled: match value.get("enabled") {
                Some(value) => value.as_bool().unwrap_or(default.enabled),
                None => default.enabled
            },

            private: match value.get("private") {
                Some(value) => match value.as_array() {
                    Some(values) => {
                        let mut private = Vec::new();

                        for value in values {
                            if let Some(server) = value.as_str() {
                                private.push(server.to_string());
                            }
                        }

                        private
                    },
                    None => default.private
                },
                None => default.private
            }
        }
    }
}
