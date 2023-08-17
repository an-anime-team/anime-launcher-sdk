pub mod wine_lang;
pub mod wine_sync;
pub mod wine_drives;
pub mod virtual_desktop;
pub mod shared_libraries;

pub mod prelude {
    pub use super::wine_drives::*;

    pub use super::wine_lang::WineLang;
    pub use super::wine_sync::WineSync;
    pub use super::virtual_desktop::VirtualDesktop;
    pub use super::shared_libraries::SharedLibraries;
}

#[macro_export]
macro_rules! config_impl_wine_schema {
    ($launcher_dir:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Wine {
            pub prefix: PathBuf,
            pub builds: PathBuf,
            pub selected: Option<String>,
            pub sync: WineSync,
            pub language: WineLang,
            pub borderless: bool,
            pub drives: WineDrives,
            pub virtual_desktop: VirtualDesktop,
            pub shared_libraries: SharedLibraries
        }

        impl Default for Wine {
            #[inline]
            fn default() -> Self {
                let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

                Self {
                    prefix: launcher_dir.join("prefix"),
                    builds: launcher_dir.join("runners"),
                    selected: None,
                    sync: WineSync::default(),
                    language: WineLang::default(),
                    borderless: false,
                    drives: WineDrives::default(),
                    virtual_desktop: VirtualDesktop::default(),
                    shared_libraries: SharedLibraries::default()
                }
            }
        }

        impl From<&JsonValue> for Wine {
            fn from(value: &JsonValue) -> Self {
                let default = Self::default();

                Self {
                    prefix: value.get("prefix")
                        .and_then(|value| value.as_str())
                        .map(PathBuf::from)
                        .unwrap_or(default.prefix),

                    builds: value.get("builds")
                        .and_then(|value| value.as_str())
                        .map(PathBuf::from)
                        .unwrap_or(default.builds),

                    selected: match value.get("selected") {
                        Some(value) => {
                            if value.is_null() {
                                None
                            } else {
                                match value.as_str() {
                                    Some(value) => Some(value.to_string()),
                                    None => default.selected
                                }
                            }
                        },
                        None => default.selected
                    },

                    sync: value.get("sync")
                        .map(WineSync::from)
                        .unwrap_or(default.sync),

                    language: value.get("language")
                        .map(WineLang::from)
                        .unwrap_or(default.language),

                    borderless: value.get("borderless")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(default.borderless),

                    drives: value.get("drives")
                        .map(WineDrives::from)
                        .unwrap_or(default.drives),

                    virtual_desktop: value.get("virtual_desktop")
                        .map(VirtualDesktop::from)
                        .unwrap_or(default.virtual_desktop),

                    shared_libraries: value.get("shared_libraries")
                        .map(SharedLibraries::from)
                        .unwrap_or(default.shared_libraries),
                }
            }
        }
    };
}
