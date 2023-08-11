use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FpsStarRail {
    // 30
    Thirty,

    // 60
    Sixty,

    // 120
    HundredTwenty
}

impl FpsStarRail {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Thirty,
            Self::Sixty,
            Self::HundredTwenty
        ]
    }

    pub fn from_num(fps: u64) -> Self {
        match fps {
            30 => Self::Thirty,
            60 => Self::Sixty,
            120 => Self::HundredTwenty,
            _   => Self::HundredTwenty
        }
    }

    pub fn to_num(&self) -> u64 {
        match self {
            Self::Thirty => 30,
            Self::Sixty => 60,
            Self::HundredTwenty     => 120
        }
    }
}