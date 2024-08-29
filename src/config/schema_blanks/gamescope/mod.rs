use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod window_mode;
pub mod window_size;
pub mod framerate;
pub mod upscaling;
pub mod options;

pub mod prelude {
    pub use super::Gamescope;
    pub use super::window_mode::GamescopeWindowMode;
    pub use super::window_size::GamescopeWindowSize;
    pub use super::framerate::GamescopeFramerate;
    pub use super::upscaling::*;
    pub use super::options::GamescopeOptions;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gamescope {
    /// Enable gamescope.
    pub enabled: bool,

    /// Game window mode.
    pub window_mode: GamescopeWindowMode,

    /// Size of the game window.
    ///
    /// ```text
    /// --nested-width
    /// --nested-height
    /// ```
    pub game_window: GamescopeWindowSize,

    /// Size of the gamescope window.
    ///
    /// ```text
    /// --output-width
    /// --output-height
    /// ```
    pub gamescope_window: GamescopeWindowSize,

    /// Game refresh rate (frames per second).
    pub framerate: GamescopeFramerate,

    /// Upscaling settings.
    pub upscaling: GamescopeUpscaling,

    /// Extra gamescope options.
    pub options: GamescopeOptions,

    /// List of extra gamescope arguments.
    pub extra_args: String
}

impl Default for Gamescope {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: false,
            window_mode: GamescopeWindowMode::default(),
            game_window: GamescopeWindowSize::default(),
            gamescope_window: GamescopeWindowSize::default(),
            framerate: GamescopeFramerate::default(),
            upscaling: GamescopeUpscaling::default(),
            options: GamescopeOptions::default(),
            extra_args: String::new()
        }
    }
}

impl From<&JsonValue> for Gamescope {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            enabled: value.get("enabled")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.enabled),

            game_window: value.get("game")
                .map(GamescopeWindowSize::from)
                .unwrap_or(default.game_window),

            gamescope_window: value.get("gamescope")
                .map(GamescopeWindowSize::from)
                .unwrap_or(default.gamescope_window),

            window_mode: value.get("window_mode")
                .map(GamescopeWindowMode::from)
                .unwrap_or(default.window_mode),

            framerate: value.get("framerate")
                .map(GamescopeFramerate::from)
                .unwrap_or(default.framerate),

            upscaling: value.get("upscaling")
                .map(GamescopeUpscaling::from)
                .unwrap_or(default.upscaling),

            options: value.get("options")
                .map(GamescopeOptions::from)
                .unwrap_or(default.options),

            extra_args: value.get("extra_args")
                .and_then(JsonValue::as_str)
                .map(String::from)
                .unwrap_or(default.extra_args)
        }
    }
}

impl Gamescope {
    pub fn get_command(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let flags = [
            String::from("gamescope"),
            self.game_window.get_command("nested"),
            self.gamescope_window.get_command("output"),
            self.window_mode.get_flag().to_string(),
            self.framerate.get_command(),
            self.upscaling.get_command(),
            self.options.get_command(),
            self.extra_args.clone()
        ];

        let flags = flags.into_iter()
            .filter(|flag| !flag.is_empty())
            .collect::<Vec<_>>();

        Some(flags.join(" "))
    }
}
