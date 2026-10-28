use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Downloader {
    /// Minimal amount of system memory in megabytes available to downloader.
    pub target_memory_usage: u64,

    /// Maximal amount of files downloader will process in parallel.
    /// `0` means unlimited.
    pub max_parallel_streams: u16,

    /// Verify files by size only instead of calculating their MD5 hashes.
    pub fast_verify: bool
}

impl Default for Downloader {
    #[inline]
    fn default() -> Self {
        Self {
            target_memory_usage: 256,
            max_parallel_streams: 64,
            fast_verify: false
        }
    }
}

impl From<&JsonValue> for Downloader {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            target_memory_usage: value
                .get("target_memory_usage")
                .and_then(JsonValue::as_u64)
                .unwrap_or(default.target_memory_usage),

            max_parallel_streams: value
                .get("max_parallel_streams")
                .and_then(JsonValue::as_u64)
                .map(|value| value.min(u16::MAX as u64) as u16)
                .unwrap_or(default.max_parallel_streams),

            fast_verify: value
                .get("fast_verify")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.fast_verify)
        }
    }
}
