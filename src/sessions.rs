use std::path::Path;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sessions<SessionData> {
    pub current: Option<String>,
    pub sessions: HashMap<String, SessionData>
}

impl<T> Default for Sessions<T> {
    #[inline]
    fn default() -> Self {
        Self {
            current: None,
            sessions: HashMap::new()
        }
    }
}

pub trait SessionsExt {
    type SessionData;

    /// Get sessions descriptor
    /// 
    /// If it doesn't exist, then default values will be returned
    fn get_sessions() -> anyhow::Result<Sessions<Self::SessionData>>;

    /// Update sessions descriptor
    fn set_sessions(sessions: Sessions<Self::SessionData>) -> anyhow::Result<()>;

    /// Get current session name
    fn get_current() -> anyhow::Result<Option<String>> {
        Ok(Self::get_sessions()?.current)
    }

    /// Set current session name
    fn set_current(name: String) -> anyhow::Result<()> {
        let mut sessions = Self::get_sessions()?;

        sessions.current = Some(name);

        Self::set_sessions(sessions)
    }

    /// List available sessions
    fn list() -> anyhow::Result<HashMap<String, Self::SessionData>> {
        Ok(Self::get_sessions()?.sessions)
    }

    /// Remove session with given name
    /// 
    /// Sets current session to `None` if its name passed
    fn remove(name: impl AsRef<str>) -> anyhow::Result<()> {
        let mut sessions = Self::get_sessions()?;

        if let Some(current) = &sessions.current {
            if current == name.as_ref() {
                sessions.current = None;
            }
        }

        sessions.sessions.remove(name.as_ref());

        Self::set_sessions(sessions)
    }

    /// Update saved session using files from the wine prefix
    fn update(name: String, prefix: impl AsRef<Path>) -> anyhow::Result<()>;

    /// Apply saved session to the wine prefix
    fn apply(name: String, prefix: impl AsRef<Path>) -> anyhow::Result<()>;
}
