use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum GamescopeUpscaleFilter {
    #[default]
    None,

    Linear,
    Nearest,
    FSR,
    NIS,
    Pixel
}

impl GamescopeUpscaleFilter {
    #[inline]
    pub fn get_flag(&self) -> &'static str {
        match self {
            Self::None    => "",
            Self::Linear  => "--filter linear",
            Self::Nearest => "--filter nearest",
            Self::FSR     => "--filter fsr",
            Self::NIS     => "--filter nis",
            Self::Pixel   => "--filter pixel"
        }
    }
}

impl From<&JsonValue> for GamescopeUpscaleFilter {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
