use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamescopeOptions {
    /// Enable HDR output (needs Gamescope WSI layer enabled for support from clients).
    ///
    /// ```text
    /// --hdr-enabled
    /// ```
    pub hdr_support: bool,

    /// Use realtime scheduling.
    ///
    /// ```text
    /// --rt
    /// ```
    pub realtime_scheduler: bool,

    /// Enable adaptive sync if available (variable rate refresh).
    ///
    /// ```text
    /// --adaptive-sync
    /// ```
    pub adaptive_sync: bool,

    /// Always use relative mouse mode instead of flipping dependent on cursor visibility.
    ///
    /// ```text
    /// --force-grab-cursor
    /// ```
    pub force_grab_cursor: bool,

    /// Enable mangohud support.
    ///
    /// ```text
    /// --mangoapp
    /// ```
    pub mangohud: bool
}

impl Default for GamescopeOptions {
    #[inline]
    fn default() -> Self {
        Self {
            hdr_support: false,
            realtime_scheduler: false,
            adaptive_sync: false,
            force_grab_cursor: false,
            mangohud: false
        }
    }
}

impl GamescopeOptions {
    pub fn get_command(&self) -> String {
        let mut flags = Vec::with_capacity(5);

        if self.hdr_support {
            flags.push("--hdr-enabled");
        }

        if self.realtime_scheduler {
            flags.push("--rt");
        }

        if self.adaptive_sync {
            flags.push("--adaptive-sync");
        }

        if self.force_grab_cursor {
            flags.push("--force-grab-cursor");
        }

        if self.mangohud {
            flags.push("--mangoapp");
        }

        flags.join(" ")
    }
}

impl From<&JsonValue> for GamescopeOptions {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            hdr_support: value.get("hdr_support")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.hdr_support),

            realtime_scheduler: value.get("realtime_scheduler")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.realtime_scheduler),

            adaptive_sync: value.get("adaptive_sync")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.adaptive_sync),

            force_grab_cursor: value.get("force_grab_cursor")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.force_grab_cursor),

            mangohud: value.get("mangohud")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.mangohud)
        }
    }
}
