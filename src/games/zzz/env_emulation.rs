use serde::{Serialize, Deserialize};
use enum_ordinalize::Ordinalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ordinalize)]
pub enum Environment {
    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=mihoyo
    /// game_version=[game version]
    /// sub_channel=0
    /// ```
    PC,

    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=pcseaepic
    /// game_version=[game version]
    /// # plugin_sdk_version=2.14.2 (??? not used now)
    /// sub_channel=3
    /// ```
    Epic,

    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=pcgoogle
    /// game_version=[game version]
    /// sub_channel=6
    /// ```
    Android
}

impl Default for Environment {
    #[inline]
    fn default() -> Self {
        Self::PC
    }
}

impl Environment {
    /// Generate `config.ini`'s content
    pub fn generate_config(&self, game_version: impl AsRef<str>) -> String {
        match self {
            Self::PC => [
                "[General]",
                "channel=1",
                "cps=mihoyo",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=0"
            ].join("\n"),

            Self::Epic => [
                "[General]",
                "channel=1",
                "cps=pcseaepic",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=3"
            ].join("\n"),

            Self::Android => [
                "[General]",
                "channel=1",
                "cps=pcgoogle",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=6"
            ].join("\n")
        }
    }
}
