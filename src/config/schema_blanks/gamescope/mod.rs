use std::process::Command;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

pub mod size;
pub mod framerate;
pub mod window_type;

pub mod prelude {
    pub use super::Gamescope;
    pub use super::size::Size;
    pub use super::framerate::Framerate;
    pub use super::window_type::WindowType;
}

use prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gamescope {
    pub enabled: bool,
    pub game: Size,
    pub gamescope: Size,
    pub framerate: Framerate,
    pub integer_scaling: bool,
    pub fsr: bool,
    pub nis: bool,
    pub window_type: WindowType,
    pub force_grab_cursor: bool,
}

impl Default for Gamescope {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: false,
            game: Size::default(),
            gamescope: Size::default(),
            framerate: Framerate::default(),
            integer_scaling: true,
            fsr: false,
            nis: false,
            window_type: WindowType::default(),
            force_grab_cursor: false
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

            game: value.get("game")
                .map(Size::from)
                .unwrap_or(default.game),

            gamescope: value.get("gamescope")
                .map(Size::from)
                .unwrap_or(default.gamescope),

            framerate: value.get("framerate")
                .map(Framerate::from)
                .unwrap_or(default.framerate),

            integer_scaling: value.get("integer_scaling")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.integer_scaling),

            fsr: value.get("fsr")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.fsr),

            nis: value.get("nis")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.nis),

            window_type: value.get("window_type")
                .map(WindowType::from)
                .unwrap_or(default.window_type),

                force_grab_cursor: value.get("force_grab_cursor")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.force_grab_cursor),
        }
    }
}

// TODO: temporary workaround for transition period, will be removed in future
#[cached::proc_macro::cached]
fn is_legacy_version() -> bool {
    // gamescope doesn't have --version, so parsing --help instead
    Command::new("gamescope").arg("--help").output()

        // if no --filter, then it's legacy version
        // also for whatever reason --help is printed to stderr
        .map(|help| !String::from_utf8_lossy(&help.stderr).contains("-F, --filter"))

        // If failed to launch gamescope, then yes, it's legacy (it's not but meh)
        .unwrap_or(true)
}

impl Gamescope {
    /// Check if available gamescope version is legacy (<3.12.0)
    pub fn is_legacy_version() -> bool {
        is_legacy_version()
    }

    pub fn get_command(&self) -> Option<String> {
        // https://github.com/bottlesdevs/Bottles/blob/b908311348ed1184ead23dd76f9d8af41ff24082/src/backend/wine/winecommand.py#L478
        // https://github.com/ValveSoftware/gamescope#options
        if self.enabled {
            let mut gamescope = String::from("gamescope");

            // Set window type
            match self.window_type {
                WindowType::Borderless => gamescope += " -b",
                WindowType::Fullscreen => gamescope += " -f"
            }

            // Set game width
            if self.game.width > 0 {
                gamescope += &format!(" -w {}", self.game.width);
            }

            // Set game height
            if self.game.height > 0 {
                gamescope += &format!(" -h {}", self.game.height);
            }

            // Set gamescope width
            if self.gamescope.width > 0 {
                gamescope += &format!(" -W {}", self.gamescope.width);
            }

            // Set gamescope height
            if self.gamescope.height > 0 {
                gamescope += &format!(" -H {}", self.gamescope.height);
            }

            // Set focused framerate limit
            if self.framerate.focused > 0 {
                gamescope += &format!(" -r {}", self.framerate.focused);
            }

            // Set unfocused framerate limit
            if self.framerate.unfocused > 0 {
                gamescope += &format!(" -o {}", self.framerate.unfocused);
            }

            // Set integer scaling
            if self.integer_scaling {
                gamescope += if Self::is_legacy_version() {
                    " -n"
                } else {
                    " -S integer"
                }
            }

            // Set FSR support
            if self.fsr {
                gamescope += if Self::is_legacy_version() {
                    " -U"
                } else {
                    " -F fsr"
                }
            }

            // Set NIS (Nvidia Image Scaling) support
            if self.nis {
                gamescope += if Self::is_legacy_version() {
                    " -Y"
                } else {
                    " -F nis"
                }
            }

            if self.force_grab_cursor {
                gamescope += " --force-grab-cursor"
            }

            Some(gamescope)
        }

        else {
            None
        }
    }
}
