use std::path::PathBuf;

pub mod schema;

pub use schema::Schema;

use crate::config::ConfigExt;
use crate::honkai::consts::config_file;

static mut CONFIG: Option<schema::Schema> = None;

pub struct Config;

impl ConfigExt for Config {
    type Schema = schema::Schema;

    #[inline]
    fn config_file() -> PathBuf {
        config_file().expect("Failed to resolve config file path")
    }

    #[inline]
    fn default_schema() -> Self::Schema {
        Self::Schema::default()
    }

    #[inline]
    fn serialize_schema(schema: Self::Schema) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(&schema)?)
    }

    #[inline]
    fn deserialize_schema<T: AsRef<str>>(schema: T) -> anyhow::Result<Self::Schema> {
        Ok(serde_json::from_str(schema.as_ref())?)
    }

    #[inline]
    fn clone_schema(schema: &Self::Schema) -> Self::Schema {
        schema.clone()
    }

    #[inline]
    fn get() -> anyhow::Result<Self::Schema> {
        unsafe {
            match &CONFIG {
                Some(config) => Ok(config.clone()),
                None => Self::get_raw()
            }
        }
    }

    #[inline]
    fn update(schema: Self::Schema) {
        unsafe {
            CONFIG = Some(schema);
        }
    }
}
