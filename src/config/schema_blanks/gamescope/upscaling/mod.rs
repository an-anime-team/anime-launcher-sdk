use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

mod upscaler;
mod filter;
mod sharpness;

pub use upscaler::GamescopeUpscaler;
pub use filter::GamescopeUpscaleFilter;
pub use sharpness::GamescopeUpscaleSharpness;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamescopeUpscaling {
    /// Upscaling algorithm.
    pub upscaler: GamescopeUpscaler,

    /// Upscaled image filter.
    pub filter: GamescopeUpscaleFilter,

    /// Upscaling sharpness.
    pub sharpness: GamescopeUpscaleSharpness
}

impl GamescopeUpscaling {
    #[inline]
    pub fn get_command(&self) -> String {
        let flags = [
            self.upscaler.get_flag(),
            self.filter.get_flag(),
            self.sharpness.get_flag()
        ];

        flags.join(" ")
    }
}

impl From<&JsonValue> for GamescopeUpscaling {
    #[inline]
    fn from(value: &JsonValue) -> Self {
        Self {
            upscaler: value.get("upscaler")
                .map(GamescopeUpscaler::from)
                .unwrap_or_default(),

            filter: value.get("filter")
                .map(GamescopeUpscaleFilter::from)
                .unwrap_or_default(),

            sharpness: value.get("sharpness")
                .map(GamescopeUpscaleSharpness::from)
                .unwrap_or_default()
        }
    }
}
