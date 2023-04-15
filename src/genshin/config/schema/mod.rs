use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "components")]
use crate::components::wine::Version as WineVersion;

#[cfg(feature = "components")]
use crate::components::dxvk::Version as DxvkVersion;

pub mod launcher;
pub mod game;
pub mod patch;

#[cfg(feature = "components")]
pub mod components;

pub mod prelude {
    pub use super::launcher::prelude::*;
    pub use super::game::prelude::*;

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

impl Schema {
    #[cfg(feature = "components")]
    /// Get selected wine version
    pub fn get_selected_wine(&self) -> anyhow::Result<Option<WineVersion>> {
        match &self.game.wine.selected {
            Some(selected) => WineVersion::find_in(&self.components.path, selected),
            None => Ok(None)
        }
    }

    #[cfg(feature = "components")]
    /// Get selected dxvk version
    pub fn get_selected_dxvk(&self) -> anyhow::Result<Option<DxvkVersion>> {
        match wincompatlib::dxvk::Dxvk::get_version(&self.game.wine.prefix)? {
            Some(version) => DxvkVersion::find_in(&self.components.path, version),
            None => Ok(None)
        }
    }
}
