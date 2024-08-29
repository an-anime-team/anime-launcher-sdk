use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum GamescopeUpscaler {
    #[default]
    None,

    Auto,
    Integer,
    Fit,
    Fill,
    Stretch
}

impl GamescopeUpscaler {
    #[inline]
    pub fn get_flag(&self) -> &'static str {
        match self {
            Self::None    => "",
            Self::Auto    => "--scaler auto",
            Self::Integer => "--scaler integer",
            Self::Fit     => "--scaler fit",
            Self::Fill    => "--scaler fill",
            Self::Stretch => "--scaler stretch"
        }
    }
}

impl From<&JsonValue> for GamescopeUpscaler {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
