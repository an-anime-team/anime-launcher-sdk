use std::path::Path;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

mod mounts;

pub use mounts::Mounts;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sandbox {
    /// Use `bwrap` to run the game. Default is `false`
    pub enabled: bool,

    /// Mount tmpfs to `/home`, `/var/home/$USER` and `$HOME`. Default is `true`
    pub isolate_home: bool,

    /// Spoof original hostname. Default is `None`
    pub hostname: Option<String>,

    /// Append additional bwrap arguments. Default is `None`
    pub args: Option<String>,

    /// List of paths to which tmpfs will be mounted. Default is empty
    pub private: Vec<String>,

    /// Maps of directories mounts
    pub mounts: Mounts
}

impl Default for Sandbox {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: false,
            isolate_home: true,
            hostname: None,
            args: None,
            private: vec![],
            mounts: Mounts::default()
        }
    }
}

impl From<&JsonValue> for Sandbox {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            enabled: match value.get("enabled") {
                Some(value) => value.as_bool().unwrap_or(default.enabled),
                None => default.enabled
            },

            isolate_home: match value.get("isolate_home") {
                Some(value) => value.as_bool().unwrap_or(default.isolate_home),
                None => default.isolate_home
            },

            hostname: match value.get("hostname") {
                Some(value) => {
                    if value.is_null() {
                        None
                    } else {
                        match value.as_str() {
                            Some(value) => Some(value.to_string()),
                            None => default.hostname
                        }
                    }
                },
                None => default.hostname
            },

            args: match value.get("args") {
                Some(value) => {
                    if value.is_null() {
                        None
                    } else {
                        match value.as_str() {
                            Some(value) => Some(value.to_string()),
                            None => default.args
                        }
                    }
                },
                None => default.args
            },

            private: match value.get("private") {
                Some(value) => match value.as_array() {
                    Some(values) => {
                        let mut private = Vec::new();

                        for value in values {
                            if let Some(server) = value.as_str() {
                                private.push(server.to_string());
                            }
                        }

                        private
                    },
                    None => default.private
                },
                None => default.private
            },

            mounts: match value.get("mounts") {
                Some(value) => Mounts::from(value),
                None => default.mounts
            }
        }
    }
}

impl Sandbox {
    /// Return `bwrap [args]` command
    /// 
    /// ### Mounts:
    /// 
    /// | Original | Mounted | Type | Optional |
    /// | :- | :- | :- | :- |
    /// | `/` | `/` | read-only bind | false |
    /// | `/tmp` | `/tmp` | bind | false |
    /// | `/proc` | `/proc` | bind | false |
    /// | `/dev` | `/dev` | dev bind | false |
    /// | - | `/home` | tmpfs | true |
    /// | - | `/var/home/$USER` | tmpfs | true |
    /// | - | `$HOME` | tmpfs | true |
    /// | `wine_dir` | `/tmp/sandbox/wine` | bind | false |
    /// | `prefix_dir` | `/tmp/sandbox/prefix` | bind | false |
    /// | `game_dir` | `/tmp/sandbox/game` | bind | false |
    /// | <mounts/read_only> | <mounts/read_only> | read-only bind | true |
    /// | <mounts/binds> | <mounts/binds> | bind | true |
    /// | <mounts/symlinks> | <mounts/symlinks> | symlink | true |
    pub fn get_command(&self, wine_dir: impl AsRef<str>, prefix_dir: impl AsRef<str>, game_dir: impl AsRef<str>) -> String {
        let mut command = String::from("bwrap --ro-bind / /");

        command.push_str(" --bind /tmp /tmp");
        command.push_str(" --bind /proc /proc");
        command.push_str(" --dev-bind /dev /dev");

        if let Some(hostname) = &self.hostname {
            command += &format!(" --hostname '{hostname}'");
        }

        if self.isolate_home {
            if Path::new("/home").is_dir() {
                command.push_str(" --tmpfs /home");
            }

            if Path::new("/var/home").is_dir() {
                command.push_str(" --tmpfs /var/home");
            }

            if let Ok(user) = std::env::var("USER") {
                let dir = format!("/var/home/{}", user.trim());

                if Path::new(&dir).is_dir() {
                    command += &format!(" --tmpfs '{dir}'");
                }
            }

            if let Ok(home) = std::env::var("HOME") {
                let dir = home.trim();

                if Path::new(&dir).is_dir() {
                    command += &format!(" --tmpfs '{dir}'");
                }
            }
        }

        for path in &self.private {
            command += &format!(" --tmpfs '{}'", path.trim());
        }

        for (from, to) in &self.mounts.read_only {
            command += &format!(" --ro-bind '{}' '{}'", from.trim(), to.trim());
        }

        for (from, to) in &self.mounts.bind {
            command += &format!(" --bind '{}' '{}'", from.trim(), to.trim());
        }

        for (from, to) in &self.mounts.symlinks {
            command += &format!(" --symlink '{}' '{}'", from.trim(), to.trim());
        }

        command += &format!(" --bind '{}' /tmp/sandbox/wine", wine_dir.as_ref());
        command += &format!(" --bind '{}' /tmp/sandbox/prefix", prefix_dir.as_ref());
        command += &format!(" --bind '{}' /tmp/sandbox/game", game_dir.as_ref());

        command.push_str(" --die-with-parent");

        // --unshare-pid breaks wine

        command.push_str(" --unshare-user");
        command.push_str(" --unshare-ipc");
        command.push_str(" --unshare-uts");
        command.push_str(" --unshare-cgroup");

        if let Some(args) = &self.args {
            command.push_str(args.trim());
        }

        command
    }
}
