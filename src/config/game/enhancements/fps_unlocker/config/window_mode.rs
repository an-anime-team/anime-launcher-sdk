use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    None,
    Popup,
    Fullscreen
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::None
    }
}

impl From<&JsonValue> for WindowMode {
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl TryFrom<u32> for WindowMode {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Popup),
            2 => Ok(Self::Fullscreen),

            _ => Err(String::from("Failed to convert number to WindowMode enum"))
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<u32> for WindowMode {
    fn into(self) -> u32 {
        match self {
            Self::None       => 0,
            Self::Popup      => 1,
            Self::Fullscreen => 2
        }
    }
}
