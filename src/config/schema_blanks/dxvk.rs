#[macro_export]
macro_rules! config_impl_dxvk_schema {
    ($launcher_dir:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Dxvk {
            pub builds: PathBuf
        }

        impl Default for Dxvk {
            #[inline]
            fn default() -> Self {
                let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

                Self {
                    builds: launcher_dir.join("dxvks")
                }
            }
        }

        impl From<&JsonValue> for Dxvk {
            fn from(value: &JsonValue) -> Self {
                let default = Self::default();

                Self {
                    builds: match value.get("builds") {
                        Some(value) => match value.as_str() {
                            Some(value) => PathBuf::from(value),
                            None => default.builds
                        },
                        None => default.builds
                    }
                }
            }
        }
    }
}
