use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordRpc {
    pub app_id: u64,
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    pub icon: String
}

// TODO: add honkers-specific discord rpc

impl Default for DiscordRpc {
    #[inline]
    fn default() -> Self {
        Self {
            app_id: 901534333360304168,
            enabled: false,

            title: String::from("Researching the world"),
            subtitle: String::from("of Teyvat"),
            icon: String::from("launcher")
        }
    }
}

impl From<&JsonValue> for DiscordRpc {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            app_id: match value.get("app_id") {
                Some(value) => value.as_u64().unwrap_or(default.app_id),
                None => default.app_id
            },

            enabled: match value.get("enabled") {
                Some(value) => value.as_bool().unwrap_or(default.enabled),
                None => default.enabled
            },

            title: match value.get("title") {
                Some(value) => value.as_str().unwrap_or(&default.title).to_string(),
                None => default.title
            },

            subtitle: match value.get("subtitle") {
                Some(value) => value.as_str().unwrap_or(&default.subtitle).to_string(),
                None => default.subtitle
            },

            icon: match value.get("icon") {
                Some(value) => value.as_str().unwrap_or(&default.icon).to_string(),
                None => default.icon
            }
        }
    }
}
