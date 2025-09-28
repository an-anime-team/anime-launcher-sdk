use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sessions::{Sessions as SessionsDescriptor, SessionsExt};
use super::consts::launcher_dir;

/// Get default sessions file path
///
/// `$HOME/.local/share/honkers-launcher/sessions.json`
#[inline]
pub fn sessions_file() -> anyhow::Result<PathBuf> {
    launcher_dir().map(|dir| dir.join("sessions.json"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionData {
    // [Software\\miHoYo\\Honkai Impact 3rd]
    pub game_reg: String,

    // [Software\\miHoYoSDK]
    pub sdk_reg: String
}

pub struct Sessions;

impl SessionsExt for Sessions {
    type SessionData = SessionData;

    fn get_sessions() -> anyhow::Result<SessionsDescriptor<Self::SessionData>> {
        let path = sessions_file()?;

        if !path.exists() {
            tracing::warn!("Session file doesn't exist. Returning default value");

            return Ok(SessionsDescriptor::default());
        }

        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    fn set_sessions(sessions: SessionsDescriptor<Self::SessionData>) -> anyhow::Result<()> {
        Ok(std::fs::write(
            sessions_file()?,
            serde_json::to_string_pretty(&sessions)?
        )?)
    }

    fn update(name: String, prefix: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut sessions = Self::get_sessions()?;

        tracing::info!(
            "Updating session '{name}' from prefix: {:?}",
            prefix.as_ref()
        );

        let mut new_session = Self::SessionData {
            game_reg: String::new(),
            sdk_reg: String::new()
        };

        for entry in std::fs::read_to_string(prefix.as_ref().join("user.reg"))?.split("\n\n") {
            if entry.starts_with("[Software\\\\miHoYo\\\\Honkai Impact 3rd]") {
                new_session.game_reg = entry.to_owned();
            }
            else if entry.starts_with("[Software\\\\miHoYoSDK]") {
                new_session.sdk_reg = entry.to_owned();
            }
        }

        sessions.sessions.insert(name, new_session);

        Self::set_sessions(sessions)
    }

    fn apply(name: String, prefix: impl AsRef<Path>) -> anyhow::Result<()> {
        let sessions = Self::get_sessions()?;

        let Some(session) = sessions.sessions.get(&name)
        else {
            anyhow::bail!("Session with given name doesn't exist");
        };

        tracing::info!("Applying session '{name}' to prefix: {:?}", prefix.as_ref());

        let entries: String = std::fs::read_to_string(prefix.as_ref().join("user.reg"))?
            .split("\n\n")
            .map(|entry| {
                let new_entry = if entry.starts_with("[Software\\\\miHoYo\\\\Honkai Impact 3rd]") {
                    session.game_reg.clone()
                }
                else if entry.starts_with("[Software\\\\miHoYoSDK]") {
                    session.sdk_reg.clone()
                }
                else {
                    entry.to_owned()
                };

                new_entry + "\n\n"
            })
            .collect();

        Ok(std::fs::write(
            prefix.as_ref().join("user.reg"),
            format!("{}\n", entries.trim_end())
        )?)
    }
}
