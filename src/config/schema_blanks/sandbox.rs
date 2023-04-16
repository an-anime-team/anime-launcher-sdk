use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sandbox {
    /// Use `bwrap` to run the game. Default is `true`
    pub enabled: bool,

    /// Mount tmpfs to `/home`, `/var/home/$USER` and `$HOME`. Default is `true`
    pub isolate_home: bool,

    /// List of paths to which tmpfs will be mounted. Default is empty
    pub private: Vec<String>
}

impl Default for Sandbox {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: false,
            isolate_home: true,
            private: vec![]
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
    /// | - | `/home` | tmpfs | true |
    /// | - | `/var/home/$USER` | tmpfs | true |
    /// | - | `$HOME` | tmpfs | true |
    /// | - | `/tmp` | tmpfs | false |
    /// | `wine_dir` | `/tmp/sandbox/wine` | bind | false |
    /// | `prefix_dir` | `/tmp/sandbox/prefix` | bind | false |
    /// | `game_dir` | `/tmp/sandbox/game` | bind | false |
    pub fn get_command(&self, wine_dir: impl AsRef<str>, prefix_dir: impl AsRef<str>, game_dir: impl AsRef<str>) -> String {
        let mut command = String::from("bwrap --ro-bind / /");

        if self.isolate_home {
            command.push_str(" --tmpfs /home");
            command.push_str(" --tmpfs /var/home");

            if let Ok(user) = std::env::var("USER") {
                command += &format!(" --tmpfs '/var/home/{}'", user.trim());
            }

            if let Ok(home) = std::env::var("HOME") {
                command += &format!(" --tmpfs '{}'", home.trim());
            }
        }

        for path in &self.private {
            command += &format!(" --tmpfs '{}'", path.trim());
        }

        command.push_str(" --tmpfs /tmp");

        command.push_str(&format!(" --bind '{}' /tmp/sandbox/wine", wine_dir.as_ref()));
        command.push_str(&format!(" --bind '{}' /tmp/sandbox/prefix", prefix_dir.as_ref()));
        command.push_str(&format!(" --bind '{}' /tmp/sandbox/game", game_dir.as_ref()));

        command.push_str(" --chdir /");
        command.push_str(" --die-with-parent");

        command.push_str(" --unshare-all");
        command.push_str(" --share-net");

        command
    }
}
