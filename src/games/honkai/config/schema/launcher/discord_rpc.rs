use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::discord_rpc::DiscordRpcParams;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordRpc {
    pub app_id: u64,
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    pub icon: String
}

impl From<DiscordRpc> for DiscordRpcParams {
    #[inline]
    fn from(config: DiscordRpc) -> Self {
        Self {
            app_id: config.app_id,
            enabled: config.enabled,
            title: config.title,
            subtitle: config.subtitle,
            icon: config.icon
        }
    }
}

impl Default for DiscordRpc {
    #[inline]
    fn default() -> Self {
        Self {
            app_id: 1015417833603219477,
            enabled: false,

            title: String::from("Fighting the"),
            subtitle: String::from("Corrupted World"),
            icon: String::from("launcher")
        }
    }
}

impl From<&JsonValue> for DiscordRpc {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        // Migration: Update old Discord RPC values
        // This will be removed in future updates

        let mut app_id = match value.get("app_id") {
            Some(value) => value.as_u64().unwrap_or(default.app_id),
            None => default.app_id
        };

        let mut title = match value.get("title") {
            Some(value) => value.as_str().unwrap_or(&default.title).to_string(),
            None => default.title
        };

        let mut subtitle = match value.get("subtitle") {
            Some(value) => value.as_str().unwrap_or(&default.subtitle).to_string(),
            None => default.subtitle
        };

        // If old values are detected - replace them by new
        if app_id == 901534333360304168 {
            app_id = defualt.app_id;
            title = default.title;
            subtitle = default.subtitle;
        }

        Self {
            app_id,

            enabled: match value.get("enabled") {
                Some(value) => value.as_bool().unwrap_or(default.enabled),
                None => default.enabled
            },

            title,
            subtitle,

            icon: match value.get("icon") {
                Some(value) => value.as_str().unwrap_or(&default.icon).to_string(),
                None => default.icon
            }
        }
    }
}
