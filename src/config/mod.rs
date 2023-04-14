use std::path::PathBuf;

/// Workpieces to create your custom config file schema
pub mod schema_blanks;

pub trait Config {
    /// Default associated config schema
    type Schema;

    /// Path to associated config file
    fn config_file() -> PathBuf;

    /// Get default config schema
    fn default_schema() -> Self::Schema;

    /// Serialize given schema
    fn serialize_schema(schema: Self::Schema) -> anyhow::Result<String>;

    /// Deserialize given schema
    fn deserialize_schema<T: AsRef<str>>(schema: T) -> anyhow::Result<Self::Schema>;

    /// Clone given schema
    fn clone_schema(schema: &Self::Schema) -> Self::Schema;

    /// Get config data
    /// 
    /// This method will load config from file once and store it into the memory.
    /// If you know that the config file was updated - you should run `get_raw` method
    /// that always loads config directly from the file. This will also update in-memory config
    fn get() -> anyhow::Result<Self::Schema>;

    /// Update in-memory config data
    /// 
    /// Use `update_raw` if you want to update config file itself
    fn update(schema: Self::Schema);

    /// Get config data
    /// 
    /// This method will always load data directly from the file and update in-memory config
    fn get_raw() -> anyhow::Result<Self::Schema> {
        tracing::debug!("Reading config data from file");

        let path = Self::config_file();

        // Try to read config if the file exists
        if path.exists() {
            let schema = Self::deserialize_schema(std::fs::read_to_string(path)?)?;

            Self::update(Self::clone_schema(&schema));

            Ok(schema)
        }

        // Otherwise create default config file
        else {
            Self::update_raw(Self::default_schema())?;

            Ok(Self::default_schema())
        }
    }

    /// Update config file
    /// 
    /// This method will also update in-memory config data
    fn update_raw(schema: Self::Schema) -> anyhow::Result<()> {
        tracing::debug!("Updating config data");

        Self::update(Self::clone_schema(&schema));

        Ok(std::fs::write(Self::config_file(), Self::serialize_schema(schema)?)?)
    }

    #[inline]
    /// Update config file from the in-memory saved config
    fn flush() -> anyhow::Result<()> {
        tracing::debug!("Flushing config data");

        Self::update_raw(Self::get()?)
    }
}
