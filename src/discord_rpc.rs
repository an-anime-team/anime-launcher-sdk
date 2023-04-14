use std::thread::JoinHandle;
use std::sync::mpsc::{self, Sender, SendError};

use discord_rich_presence::{
    activity::*,
    DiscordIpc,
    DiscordIpcClient
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordRpcParams {
    pub app_id: u64,
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    pub icon: String
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
        icon: String
    },

    /// Clear RPC activity
    ClearActivity
}

pub struct DiscordRpc {
    _thread: JoinHandle<()>,
    sender: Sender<RpcUpdates>
}

impl DiscordRpc {
    pub fn new(mut params: DiscordRpcParams) -> Self {
        let (sender, receiver) = mpsc::channel();

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

                                client.connect().expect("Failed to connect to discord");

                                client.set_activity(Self::get_activity(&params))
                                    .expect("Failed to update discord rpc activity");
                            }
                        }

                        RpcUpdates::Disconnect => {
                            if connected {
                                connected = false;

                                client.close().expect("Failed to disconnect from discord");
                            }
                        }

                        RpcUpdates::UpdateActivity { title, subtitle, icon } => {
                            params.title = title;
                            params.subtitle = subtitle;
                            params.icon = icon;

                            if connected {
                                client.set_activity(Self::get_activity(&params))
                                    .expect("Failed to update discord rpc activity");
                            }
                        }

                        RpcUpdates::ClearActivity => {
                            if connected {
                                client.clear_activity().expect("Failed to clear discord rpc activity");
                            }
                        }
                    }
                }
            }),
            sender
        }
    }

    pub fn get_activity(config: &DiscordRpcParams) -> Activity {
        Activity::new()
            .details(&config.title)
            .state(&config.subtitle)
            .assets(Assets::new().large_image(&config.icon))
    }

    #[inline]
    pub fn update(&self, update: RpcUpdates) -> Result<(), SendError<RpcUpdates>> {
        self.sender.send(update)
    }
}

impl Drop for DiscordRpc {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.update(RpcUpdates::Disconnect);
    }
}
