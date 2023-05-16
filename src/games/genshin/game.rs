use std::process::{Command, Stdio};
use std::path::PathBuf;

use anime_game_core::prelude::*;
use anime_game_core::genshin::telemetry;
use anime_game_core::genshin::game::Game;

use crate::config::ConfigExt;
use crate::genshin::config::Config;

use crate::genshin::consts;

#[cfg(feature = "fps-unlocker")]
use super::fps_unlocker::FpsUnlocker;

#[cfg(feature = "discord-rpc")]
use crate::discord_rpc::*;

#[cfg(feature = "sessions")]
use crate::sessions::SessionsExt;

#[cfg(feature = "sessions")]
use crate::genshin::sessions::Sessions;

#[derive(Debug, Clone)]
struct Folders {
    pub wine: PathBuf,
    pub prefix: PathBuf,
    pub game: PathBuf,
    pub temp: PathBuf
}

fn replace_keywords(command: impl ToString, folders: &Folders) -> String {
    command.to_string()
        .replace("%build%", folders.wine.to_str().unwrap())
        .replace("%prefix%", folders.prefix.to_str().unwrap())
        .replace("%temp%", folders.game.to_str().unwrap())
        .replace("%launcher%", &consts::launcher_dir().unwrap().to_string_lossy())
        .replace("%game%", folders.temp.to_str().unwrap())
}

/// Try to run the game
/// 
/// This function will freeze thread it was called from while the game is running
#[tracing::instrument(level = "info", ret)]
pub fn run() -> anyhow::Result<()> {
    tracing::info!("Preparing to run the game");

    let config = Config::get()?;

    let game_path = config.game.path.for_edition(config.launcher.edition);

    if !game_path.exists() {
        return Err(anyhow::anyhow!("Game is not installed"));
    }

    let Some(wine) = config.get_selected_wine()? else {
        anyhow::bail!("Couldn't find wine executable");
    };

    let features = wine.features(&config.components.path)?.unwrap_or_default();

    let mut folders = Folders {
        wine: config.game.wine.builds.join(&wine.name),
        prefix: config.game.wine.prefix.clone(),
        game: config.game.path.for_edition(config.launcher.edition).to_path_buf(),
        temp: config.launcher.temp.clone().unwrap_or(std::env::temp_dir())
    };

    // Check telemetry servers

    tracing::info!("Checking telemetry");

    if let Ok(Some(server)) = telemetry::is_disabled() {
        return Err(anyhow::anyhow!("Telemetry server is not disabled: {server}"));
    }

    // Prepare fps unlocker
    // 1) Download if needed
    // 2) Generate config file
    // 3) Generate fpsunlocker.bat from launcher.bat

    #[cfg(feature = "fps-unlocker")]
    if config.game.enhancements.fps_unlocker.enabled {
        tracing::info!("Preparing FPS unlocker");

        let unlocker = match FpsUnlocker::from_dir(&config.game.enhancements.fps_unlocker.path) {
            Ok(Some(unlocker)) => unlocker,

            other => {
                // Ok(None) means unknown version, so we should delete it before downloading newer one
                // because otherwise downloader will try to continue downloading "partially downloaded" file
                if let Ok(None) = other {
                    std::fs::remove_file(FpsUnlocker::get_binary_in(&config.game.enhancements.fps_unlocker.path))?;
                }

                tracing::info!("Unlocker is not downloaded. Downloading");

                match FpsUnlocker::download(&config.game.enhancements.fps_unlocker.path) {
                    Ok(unlocker) => unlocker,
                    Err(err) => return Err(anyhow::anyhow!("Failed to download FPS unlocker: {err}"))
                }
            }
        };

        // Generate FPS unlocker config file
        if let Err(err) = unlocker.update_config(config.game.enhancements.fps_unlocker.config) {
            return Err(anyhow::anyhow!("Failed to update FPS unlocker config: {err}"));
        }

        let bat_path = game_path.join("fps_unlocker.bat");
        let original_bat_path = game_path.join("launcher.bat");

        // Generate fpsunlocker.bat from launcher.bat
        std::fs::write(bat_path, std::fs::read_to_string(original_bat_path)?
            .replace("start GenshinImpact.exe %*", &format!("start GenshinImpact.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy()))
            .replace("start YuanShen.exe %*", &format!("start YuanShen.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy())))?;
    }

    // Generate `config.ini` if environment emulation feature is presented

    #[cfg(feature = "environment-emulation")] {
        let game = Game::new(game_path);

        std::fs::write(
            game_path.join("config.ini"),
            config.launcher.environment.generate_config(game.get_version()?.to_string())
        )?;
    }

    // Prepare bash -c '<command>'

    let mut bash_command = String::new();
    let mut windows_command = String::new();

    if config.game.enhancements.gamemode {
        bash_command += "gamemoderun ";
    }

    let run_command = features.command
        .map(|command| replace_keywords(command, &folders))
        .unwrap_or(format!("'{}'", folders.wine.join(wine.files.wine64.unwrap_or(wine.files.wine)).to_string_lossy()));

    bash_command += &run_command;
    bash_command += " ";

    if let Some(virtual_desktop) = config.game.wine.virtual_desktop.get_command("an_anime_game") {
        windows_command += &virtual_desktop;
        windows_command += " ";
    }

    windows_command += if config.game.enhancements.fps_unlocker.enabled && cfg!(feature = "fps-unlocker") {
        "fps_unlocker.bat "
    } else {
        "launcher.bat "
    };

    if config.game.wine.borderless {
        windows_command += "-screen-fullscreen 0 -popupwindow ";
    }

    // https://notabug.org/Krock/dawn/src/master/TWEAKS.md
    if config.game.enhancements.fsr.enabled {
        windows_command += "-window-mode exclusive ";
    }

    // gamescope <params> -- <command to run>
    if let Some(gamescope) = config.game.enhancements.gamescope.get_command() {
        bash_command = format!("{gamescope} -- {bash_command}");
    }

    // bwrap <params> -- <command to run>
    #[cfg(feature = "sandbox")]
    if config.sandbox.enabled {
        let bwrap = config.sandbox.get_command(
            folders.wine.to_str().unwrap(),
            folders.prefix.to_str().unwrap(),
            folders.game.to_str().unwrap()
        );

        let sandboxed_folders = Folders {
            wine: PathBuf::from("/tmp/sandbox/wine"),
            prefix: PathBuf::from("/tmp/sandbox/prefix"),
            game: PathBuf::from("/tmp/sandbox/game"),
            temp: PathBuf::from("/tmp")
        };

        bash_command = bash_command
            .replace(folders.wine.to_str().unwrap(), sandboxed_folders.wine.to_str().unwrap())
            .replace(folders.prefix.to_str().unwrap(), sandboxed_folders.prefix.to_str().unwrap())
            .replace(folders.game.to_str().unwrap(), sandboxed_folders.game.to_str().unwrap())
            .replace(folders.temp.to_str().unwrap(), sandboxed_folders.temp.to_str().unwrap());

        bash_command = format!("{bwrap} --chdir /tmp/sandbox/game -- {bash_command}");
        folders = sandboxed_folders;
    }

    // Bundle all windows arguments used to run the game into a single file
    if features.compact_launch {
        std::fs::write(folders.game.join("compact_launch.bat"), format!("start {windows_command}\nexit"))?;

        windows_command = String::from("compact_launch.bat");
    }

    // Finalize launching command
    bash_command = match &config.game.command {
        // Use user-given launch command
        Some(command) => replace_keywords(command, &folders)
            .replace("%command%", &format!("{bash_command} {windows_command}"))
            .replace("%bash_command%", &bash_command)
            .replace("%windows_command%", &windows_command),

        // Combine bash and windows parts of the command
        None => format!("{bash_command} {windows_command}")
    };

    let mut command = Command::new("bash");

    command.arg("-c");
    command.arg(&bash_command);

    // Setup environment

    command.env("WINEARCH", "win64");
    command.env("WINEPREFIX", &folders.prefix);

    // Add environment flags for selected wine
    for (key, value) in features.env.into_iter() {
        command.env(key, replace_keywords(value, &folders));
    }

    // Add environment flags for selected dxvk
    if let Ok(Some(dxvk )) = config.get_selected_dxvk() {
        if let Ok(Some(features)) = dxvk.features(&config.components.path) {
            for (key, value) in features.env.iter() {
                command.env(key, replace_keywords(value, &folders));
            }
        }
    }

    command.envs(config.game.wine.sync.get_env_vars());
    command.envs(config.game.enhancements.hud.get_env_vars(config.game.enhancements.gamescope.enabled));
    command.envs(config.game.enhancements.fsr.get_env_vars());
    command.envs(config.game.wine.language.get_env_vars());

    command.envs(config.game.environment);

    #[cfg(feature = "sessions")]
    if let Some(current) = Sessions::get_current()? {
        Sessions::apply(current, &folders.prefix)?;
    }

    // Run command

    let variables = command
        .get_envs()
        .map(|(key, value)| format!("{}=\"{}\"", key.to_string_lossy(), value.unwrap_or_default().to_string_lossy()))
        .fold(String::new(), |acc, env| acc + " " + &env);

    tracing::info!("Running the game with command: {variables} bash -c \"{bash_command}\"");

    // We use real current dir here because sandboxed one
    // obviously doesn't exist
    command.current_dir(config.game.path.for_edition(config.launcher.edition))
        .spawn()?.wait_with_output()?;

    #[cfg(feature = "discord-rpc")]
    let rpc = if config.launcher.discord_rpc.enabled {
        Some(DiscordRpc::new(config.launcher.discord_rpc.into()))
    } else {
        None
    };

    #[cfg(feature = "discord-rpc")]
    if let Some(rpc) = &rpc {
        rpc.update(RpcUpdates::Connect)?;
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3));

        let output = Command::new("ps").arg("-A").stdout(Stdio::piped()).output()?;
        let output = String::from_utf8_lossy(&output.stdout);

        if !output.contains("GenshinImpact.e") && !output.contains("unlocker.exe") {
            break;
        }
    }

    #[cfg(feature = "discord-rpc")]
    if let Some(rpc) = &rpc {
        rpc.update(RpcUpdates::Disconnect)?;
    }

    #[cfg(feature = "sessions")]
    if let Some(current) = Sessions::get_current()? {
        Sessions::update(current, folders.prefix)?;
    }

    Ok(())
}
