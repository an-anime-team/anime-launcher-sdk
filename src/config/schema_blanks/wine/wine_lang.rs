use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum WineLang {
    System,
    English,
    Russian,
    German,
    Portuguese,
    Polish,
    French,
    Spanish,
    Chinese,
    Japanese,
    Korean
}

impl Default for WineLang {
    #[inline]
    fn default() -> Self {
        Self::System
    }
}

impl From<&JsonValue> for WineLang {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl WineLang {
    /// Get environment variables corresponding to used wine language
    pub fn get_env_vars(&self) -> HashMap<&str, &str> {
        let lang = match self {
            Self::System => return HashMap::new(),

            Self::English    => "en_US.UTF-8",
            Self::Russian    => "ru_RU.UTF-8",
            Self::German     => "de_DE.UTF-8",
            Self::Portuguese => "pt_PT.UTF-8",
            Self::Polish     => "pl_PL.UTF-8",
            Self::French     => "fr_FR.UTF-8",
            Self::Spanish    => "es_ES.UTF-8",
            Self::Chinese    => "zh_CN.UTF-8",
            Self::Japanese   => "ja_JP.UTF-8",
            Self::Korean     => "ko_KR.UTF-8"
        };

        HashMap::from([
            ("LANG", lang),
            ("LC_ALL", lang)
        ])
    }
}

impl std::fmt::Display for WineLang {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}
