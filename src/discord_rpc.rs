use std::thread::JoinHandle;
use std::sync::mpsc::{self, Sender, SendError};

use serde::{Serialize, Deserialize};

use anime_game_core::minreq;

use discord_rich_presence::{
    activity::*,
    DiscordIpc,
    DiscordIpcClient
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordRpcAsset {
    pub app_id: u64,
    pub id: String,
    pub r#type: u64,
    pub name: String
}

impl DiscordRpcAsset {
    #[inline]
    pub fn get_uri(&self) -> String {
        format!("https://cdn.discordapp.com/app-assets/{}/{}.png", self.app_id, self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordRpcParams {
    pub app_id: u64,
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub start_timestamp: Option<i64>,
    pub end_timestamp: Option<i64>
}

#[derive(Debug, Clone)]
pub enum RpcUpdates {
    /// Establish RPC connection
    Connect,

    /// Terminate RPC connection. Panics if not connected
    Disconnect,

    /// Update RPC activity
    UpdateActivity {
        title: String,
        subtitle: String,
        icon: String,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>
    },

    /// Update RPC connection with already set activity params
    Update,

    /// Clear RPC activity
    ClearActivity,

    /// Reconnect RPC connection
    Reconnect
}

pub struct DiscordRpc {
    _thread: JoinHandle<()>,
    sender: Sender<RpcUpdates>
}

impl DiscordRpc {
    pub fn new(mut params: DiscordRpcParams) -> Self {
        let (sender, receiver) = mpsc::channel();
        let sender_clone = sender.clone();

        Self {
            _thread: std::thread::spawn(move || {
                let mut client = DiscordIpcClient::new(&params.app_id.to_string())
                    .expect("Failed to register discord ipc client");

                let mut connected = false;

                while let Ok(update) = receiver.recv() {
                    match update {
                        RpcUpdates::Connect => {
                            if !connected {
                                connected = true;

                                if let Err(err) = client.connect() {
                                    eprintln!("Failed to connect to discord: {:?}", err);
                                    continue;
                                }

                                if let Err(err) = client.set_activity(Self::get_activity(&params)) {
                                    eprintln!("Failed to update discord rpc activity: {:?}", err);
                                    continue;
                                }
                            }
                        }

                        RpcUpdates::Disconnect => {
                            if connected {
                                connected = false;

                                if let Err(err) = client.close() {
                                    eprintln!("Failed to disconnect from discord: {:?}", err);
                                }
                            }
                        }

                        RpcUpdates::UpdateActivity { title, subtitle, icon, start_timestamp, end_timestamp } => {
                            params.title = title;
                            params.subtitle = subtitle;
                            params.icon = icon;
                            params.start_timestamp = start_timestamp;
                            params.end_timestamp = end_timestamp;

                            if connected {
                                if let Err(err) = client.set_activity(Self::get_activity(&params)) {
                                    eprintln!("Failed to update discord rpc activity: {:?}", err);
                                    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                                        if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                                            if let Err(err) = sender_clone.send(RpcUpdates::Reconnect) {
                                                eprintln!("Failed to send reconnect message: {:?}", err);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        RpcUpdates::Update => {
                            if connected {
                                if let Err(err) = client.set_activity(Self::get_activity(&params)) {
                                    eprintln!("Failed to update discord rpc activity: {:?}", err);
                                    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                                        if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                                            if let Err(err) = sender_clone.send(RpcUpdates::Reconnect) {
                                                eprintln!("Failed to send reconnect message: {:?}", err);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        RpcUpdates::ClearActivity => {
                            if connected {
                                if let Err(err) = client.clear_activity() {
                                    eprintln!("Failed to clear discord rpc activity: {:?}", err);
                                    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                                        if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                                            if let Err(err) = sender_clone.send(RpcUpdates::Reconnect) {
                                                eprintln!("Failed to send reconnect message: {:?}", err);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        RpcUpdates::Reconnect => {
                            if connected {
                                if let Err(err) = client.close() {
                                    eprintln!("Failed to disconnect from discord: {:?}", err);
                                }
                            }

                            if let Err(err) = client.connect() {
                                eprintln!("Failed to reconnect to discord: {:?}", err);
                                continue;
                            }

                            if let Err(err) = client.set_activity(Self::get_activity(&params)) {
                                eprintln!("Failed to update discord rpc activity: {:?}", err);
                                continue;
                            }

                            connected = true;
                        }
                    }
                }
            }),
            sender
        }
    }

    pub fn get_activity(config: &DiscordRpcParams) -> Activity {
        let mut activity = Activity::new()
            .details(&config.title)
            .state(&config.subtitle)
            .assets(Assets::new().large_image(&config.icon));

        if let Some(start) = config.start_timestamp {
            activity = activity.timestamps(Timestamps::new().start(start));
        }

        if let Some(end) = config.end_timestamp {
            activity = activity.timestamps(Timestamps::new().end(end));
        }

        activity
    }

    #[inline]
    pub fn update(&self, update: RpcUpdates) -> Result<(), SendError<RpcUpdates>> {
        self.sender.send(update)
    }

    pub fn reconnect(&self) -> Result<(), SendError<RpcUpdates>> {
        self.sender.send(RpcUpdates::Reconnect)
    }

    pub fn get_assets(app_id: u64) -> anyhow::Result<Vec<DiscordRpcAsset>> {
        Ok(minreq::get(format!("https://discord.com/api/v9/oauth2/applications/{app_id}/assets"))
            .send()?
            .json::<Vec<serde_json::Value>>()?
            .into_iter()
            .map(|value| DiscordRpcAsset {
                app_id,
                id: value["id"].as_str().unwrap().to_string(),
                r#type: value["type"].as_u64().unwrap(),
                name: value["name"].as_str().unwrap().to_string()
            })
            .collect())
    }
}

impl Drop for DiscordRpc {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.update(RpcUpdates::Disconnect);
    }
}
// ihate this
