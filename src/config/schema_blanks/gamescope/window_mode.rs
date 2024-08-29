use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum GamescopeWindowMode {
    #[default]
    /// No special window settings.
    Default,

    /// Borderless window.
    ///
    /// ```text
    /// --borderless
    /// ```
    Borderless,

    /// Headless window (no window, no DRM output).
    ///
    /// ```text
    /// --headless
    /// ```
    Headless,

    /// Fullscreen window.
    ///
    /// ```text
    /// --fullscreen
    /// ```
    Fullscreen
}

impl GamescopeWindowMode {
    #[inline]
    pub fn get_flag(&self) -> &'static str {
        match self {
            Self::Default    => "",
            Self::Borderless => "--borderless",
            Self::Headless   => "--headless",
            Self::Fullscreen => "--fullscreen"
        }
    }
}

impl From<&JsonValue> for GamescopeWindowMode {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
