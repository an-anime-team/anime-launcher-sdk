use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod launcher;
pub mod game;
pub mod patch;

#[cfg(feature = "components")]
pub mod components;

pub mod prelude {
    pub use super::launcher::*;
    pub use super::game::*;
    pub use super::patch::*;

    #[cfg(feature = "components")]
    pub use super::components::*;
}

use prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    pub launcher: Launcher,
    pub game: Game,

    #[cfg(feature = "components")]
    pub components: Components,

    pub patch: Patch
}

impl From<&JsonValue> for Schema {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            launcher: match value.get("launcher") {
                Some(value) => Launcher::from(value),
                None => default.launcher
            },

            game: match value.get("game") {
                Some(value) => Game::from(value),
                None => default.game
            },

            components: match value.get("components") {
                Some(value) => Components::from(value),
                None => default.components
            },

            patch: match value.get("patch") {
                Some(value) => Patch::from(value),
                None => default.patch
            },
        }
    }
}
