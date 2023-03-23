use std::process::{Command, Stdio};

use anime_game_core::genshin::telemetry;

use super::consts;
use super::config;

#[cfg(feature = "fps-unlocker")]
use super::fps_unlocker::FpsUnlocker;

#[cfg(feature = "discord-rpc")]
use super::discord_rpc::*;

fn replace_keywords<T: ToString>(command: T, config: &config::Config) -> String {
    let wine_build = config.game.wine.builds.join(config.game.wine.selected.as_ref().unwrap());

    command.to_string()
        .replace("%build%", &wine_build.to_string_lossy())
        .replace("%prefix%", &config.game.wine.prefix.to_string_lossy())
        .replace("%temp%", &config.launcher.temp.as_ref().unwrap_or(&std::env::temp_dir()).to_string_lossy())
        .replace("%launcher%", &consts::launcher_dir().unwrap().to_string_lossy())
        .replace("%game%", &config.game.path.to_string_lossy())
}

/// Try to run the game
/// 
/// This function will freeze thread it was called from while the game is running
#[tracing::instrument(level = "info", ret)]
pub fn run() -> anyhow::Result<()> {
    tracing::info!("Preparing to run the game");

    let config = config::get()?;

    if !config.game.path.exists() {
        return Err(anyhow::anyhow!("Game is not installed"));
    }

    let Some(wine) = config.get_selected_wine()? else {
        anyhow::bail!("Couldn't find wine executable");
    };

    let features = wine.features(&config.components.path)?.unwrap_or_default();

    // Check telemetry servers

    tracing::info!("Checking telemetry");

    if let Some(server) = telemetry::is_disabled(consts::TELEMETRY_CHECK_TIMEOUT) {
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

        let bat_path = config.game.path.join("fps_unlocker.bat");
        let original_bat_path = config.game.path.join("launcher.bat");

        // Generate fpsunlocker.bat from launcher.bat
        std::fs::write(bat_path, std::fs::read_to_string(original_bat_path)?
            .replace("start GenshinImpact.exe %*", &format!("start GenshinImpact.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy()))
            .replace("start YuanShen.exe %*", &format!("start YuanShen.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy())))?;
    }

    // Prepare bash -c '<command>'

    let mut bash_command = String::new();
    let mut windows_command = String::new();

    if config.game.enhancements.gamemode {
        bash_command += "gamemoderun ";
    }

    let wine_build = config.game.wine.builds.join(&wine.name);

    let run_command = features.command
        .map(|command| replace_keywords(command, &config))
        .unwrap_or(format!("'{}'", wine_build.join(wine.files.wine64.unwrap_or(wine.files.wine)).to_string_lossy()));

    bash_command += &run_command;
    bash_command += " ";

    if let Some(virtual_desktop) = config.game.wine.virtual_desktop.get_command() {
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

    // Bundle all windows arguments used to run the game into a single file
    if features.compact_launch {
        std::fs::write(config.game.path.join("compact_launch.bat"), format!("start {}\nexit", windows_command))?;

        windows_command = String::from("compact_launch.bat");
    }

    let bash_command = match &config.game.command {
        Some(command) => replace_keywords(command, &config).replace("%command%", &bash_command),
        None => bash_command
    } + &windows_command;

    let mut command = Command::new("bash");

    command.arg("-c");
    command.arg(&bash_command);

    // Setup environment

    command.env("WINEARCH", "win64");
    command.env("WINEPREFIX", &config.game.wine.prefix);

    // Add environment flags for selected wine
    for (key, value) in features.env.into_iter() {
        command.env(key, replace_keywords(value, &config));
    }

    // Add environment flags for selected dxvk
    if let Ok(Some(dxvk )) = config.get_selected_dxvk() {
        if let Some(features) = &dxvk.features {
            for (key, value) in features.env.iter() {
                command.env(key, replace_keywords(value, &config));
            }
        }

        else if let Ok(Some(group)) = dxvk.find_group(&config.components.path) {
            for (key, value) in group.features.env.into_iter() {
                command.env(key, replace_keywords(value, &config));
            }
        }
    }

    command.envs(config.game.wine.sync.get_env_vars());
    command.envs(config.game.enhancements.hud.get_env_vars(&config));
    command.envs(config.game.enhancements.fsr.get_env_vars());
    command.envs(config.game.wine.language.get_env_vars());

    command.envs(config.game.environment);

    // Run command

    let variables = command
        .get_envs()
        .map(|(key, value)| format!("{}=\"{}\"", key.to_string_lossy(), value.unwrap_or_default().to_string_lossy()))
        .fold(String::new(), |acc, env| acc + " " + &env);

    tracing::info!("Running the game with command: {variables} bash -c \"{bash_command}\"");

    command.current_dir(config.game.path).spawn()?.wait_with_output()?;

    #[cfg(feature = "discord-rpc")]
    let rpc = if config.launcher.discord_rpc.enabled {
        Some(DiscordRpc::new(config.launcher.discord_rpc))
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

    Ok(())
}
