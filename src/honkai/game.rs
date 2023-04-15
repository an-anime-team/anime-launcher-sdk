use std::process::{Command, Stdio};

use anime_game_core::honkai::telemetry;

use crate::config::ConfigExt;
use crate::honkai::config::{Config, Schema};

use crate::honkai::consts;

#[cfg(feature = "discord-rpc")]
use crate::discord_rpc::*;

fn replace_keywords<T: ToString>(command: T, config: &Schema) -> String {
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

    let config = Config::get()?;

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

    if let Some(virtual_desktop) = config.game.wine.virtual_desktop.get_command("honkers") {
        windows_command += &virtual_desktop;
        windows_command += " ";
    }

    windows_command += "launcher.bat ";

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
        std::fs::write(config.game.path.join("compact_launch.bat"), format!("start {windows_command}\nexit"))?;

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
        if let Ok(Some(features)) = dxvk.features(&config.components.path) {
            for (key, value) in features.env.iter() {
                command.env(key, replace_keywords(value, &config));
            }
        }
    }

    command.envs(config.game.wine.sync.get_env_vars());
    command.envs(config.game.enhancements.hud.get_env_vars(config.game.enhancements.gamescope.enabled));
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

    Ok(())
}
