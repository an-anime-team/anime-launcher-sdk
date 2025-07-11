use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SophonConfig {
    pub threads: u32
}

impl Default for SophonConfig {
    #[inline]
    fn default() -> Self {
        Self {
            threads: 4
        }
    }
}

impl From<&Json> for SophonConfig {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            threads: value.get("threads")
                .and_then(Json::as_u64)
                .map(|threads| threads as u32)
                .unwrap_or(default.threads)
        }
    }
}
