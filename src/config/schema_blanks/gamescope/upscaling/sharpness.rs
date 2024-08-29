use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum GamescopeUpscaleSharpness {
    #[default]
    None,

    Smallest,
    Small,
    Balanced,
    High,
    Highest
}

impl GamescopeUpscaleSharpness {
    #[inline]
    pub fn get_flag(&self) -> &'static str {
        match self {
            Self::None     => "",
            Self::Smallest => "--sharpness 20",
            Self::Small    => "--sharpness 15",
            Self::Balanced => "--sharpness 10",
            Self::High     => "--sharpness 5",
            Self::Highest  => "--sharpness 0"
        }
    }
}

impl From<&JsonValue> for GamescopeUpscaleSharpness {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}
