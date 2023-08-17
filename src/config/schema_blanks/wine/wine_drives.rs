use std::path::{Path, PathBuf};
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllowedDrives {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z
}

impl AllowedDrives {
    #[inline]
    pub fn list() -> &'static [Self] {
        &[
            Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H, Self::I, Self::J, Self::K, Self::L, Self::M,
            Self::N, Self::O, Self::P, Self::Q, Self::R, Self::S, Self::T, Self::U, Self::V, Self::W, Self::X, Self::Y, Self::Z
        ]
    }

    /// Get unix drive name
    /// 
    /// ```
    /// assert_eq!(AllowedDrives::F, "f:");
    /// ```
    pub fn to_drive(&self) -> &str {
        match self {
            Self::A => "a:",
            Self::B => "b:",
            Self::C => "c:",
            Self::D => "d:",
            Self::E => "e:",
            Self::F => "f:",
            Self::G => "g:",
            Self::H => "h:",
            Self::I => "i:",
            Self::J => "j:",
            Self::K => "k:",
            Self::L => "l:",
            Self::M => "m:",
            Self::N => "n:",
            Self::O => "o:",
            Self::P => "p:",
            Self::Q => "q:",
            Self::R => "r:",
            Self::S => "s:",
            Self::T => "t:",
            Self::U => "u:",
            Self::V => "v:",
            Self::W => "w:",
            Self::X => "x:",
            Self::Y => "y:",
            Self::Z => "z:"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WineDrives {
    /// Symlink prefix's `drive_c` folder to the `c:` folder in the `dosdevices`
    pub drive_c: bool,

    /// If `Some`, then symlink game folder to the given folder in the `dosdevices`
    pub game_folder: Option<AllowedDrives>,

    /// Symlink paths to the given letters in the `dosdevices`
    pub map_folders: HashMap<AllowedDrives, PathBuf>
}

impl WineDrives {
    /// Automatically map all the configured folders
    pub fn map_folders(&self, game_folder: impl Into<PathBuf>, prefix_folder: impl Into<PathBuf>) -> anyhow::Result<()> {
        let mut drives = self.map_folders.clone();

        let game_folder = game_folder.into();
        let prefix_folder = prefix_folder.into();

        if self.drive_c {
            drives.insert(AllowedDrives::C, prefix_folder.join("drive_c"));
        }

        if let Some(drive) = self.game_folder {
            drives.insert(drive, game_folder);
        }

        for (drive, path) in drives {
            Self::map_folder(&prefix_folder, drive, path)?;
        }

        Ok(())
    }

    /// Map specific folder
    pub fn map_folder(prefix_folder: impl AsRef<Path>, drive: AllowedDrives, symlink_folder: impl AsRef<Path>) -> anyhow::Result<()> {
        let drive_folder = prefix_folder.as_ref()
            .join("dosdevices")
            .join(drive.to_drive());

        if drive_folder.exists() {
            std::fs::remove_file(&drive_folder)?;
        }

        std::os::unix::fs::symlink(symlink_folder.as_ref(), drive_folder)?;

        Ok(())
    }
}

impl Default for WineDrives {
    #[inline]
    fn default() -> Self {
        Self {
            drive_c: true,
            game_folder: Some(AllowedDrives::G),
            map_folders: HashMap::new()
        }
    }
}

impl From<&JsonValue> for WineDrives {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            drive_c: value.get("drive_c")
                .and_then(JsonValue::as_bool)
                .unwrap_or(default.drive_c),

            game_folder: value.get("game_folder")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.game_folder),

            map_folders: match value.get("map_folders") {
                Some(value) => match value.as_object() {
                    Some(values) => {
                        let mut drives = HashMap::new();

                        for (drive, path) in values {
                            let drive = serde_json::from_str::<AllowedDrives>(drive);
                            let path = path.as_str();

                            if let (Ok(drive), Some(path)) = (drive, path) {
                                drives.insert(drive, PathBuf::from(path));
                            }
                        }

                        drives
                    },
                    None => default.map_folders
                },
                None => default.map_folders
            },
        }
    }
}
