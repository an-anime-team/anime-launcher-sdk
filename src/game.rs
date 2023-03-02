use std::process::{Command, Stdio};

use anime_game_core::genshin::telemetry;

use super::consts;
use super::config;

#[cfg(feature = "fps-unlocker")]
use super::fps_unlocker::FpsUnlocker;

#[cfg(feature = "discord-rpc")]
use super::discord_rpc::*;

/// Try to run the game
/// 
/// If `debug = true`, then the game will be run in the new terminal window
#[tracing::instrument(level = "info", ret)]
pub fn run() -> anyhow::Result<()> {
    tracing::info!("Preparing to run the game");

    let config = config::get()?;

    if !config.game.path.exists() {
        return Err(anyhow::anyhow!("Game is not installed"));
    }

    let wine_executable = match config.try_get_wine_executable() {
        Some(path) => path,
        None => return Err(anyhow::anyhow!("Couldn't find wine executable"))
    };

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

        let bat_path = config.game.path.join("fpsunlocker.bat");
        let original_bat_path = config.game.path.join("launcher.bat");

        // Generate fpsunlocker.bat from launcher.bat
        std::fs::write(bat_path, std::fs::read_to_string(original_bat_path)?
            .replace("start GenshinImpact.exe %*", &format!("start GenshinImpact.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy()))
            .replace("start YuanShen.exe %*", &format!("start YuanShen.exe %*\n\nZ:\ncd \"{}\"\nstart unlocker.exe", unlocker.dir().to_string_lossy())))?;
    }

    // Prepare bash -c '<command>'

    let mut bash_chain = String::new();

    if config.game.enhancements.gamemode {
        bash_chain += "gamemoderun ";
    }

    bash_chain += &format!("'{}' ", wine_executable.to_string_lossy());

    if let Some(virtual_desktop) = config.game.wine.virtual_desktop.get_command() {
        bash_chain += &format!("{virtual_desktop} ");
    }

    bash_chain += if config.game.enhancements.fps_unlocker.enabled && cfg!(feature = "fps-unlocker") {
        "fpsunlocker.bat "
    } else {
        "launcher.bat "
    };

    if config.game.wine.borderless {
        bash_chain += "-screen-fullscreen 0 -popupwindow ";
    }

    // https://notabug.org/Krock/dawn/src/master/TWEAKS.md
    if config.game.enhancements.fsr.enabled {
        bash_chain += "-window-mode exclusive ";
    }

    // gamescope <params> -- <command to run>
    if let Some(gamescope) = config.game.enhancements.gamescope.get_command() {
        bash_chain = format!("{gamescope} -- {bash_chain}");
    }

    let bash_chain = match &config.game.command {
        Some(command) => command.replace("%command%", &bash_chain),
        None => bash_chain
    };

    let mut command = Command::new("bash");

    command.arg("-c");
    command.arg(&bash_chain);

    // Setup environment

    command.env("WINEARCH", "win64");
    command.env("WINEPREFIX", &config.game.wine.prefix);

    // Add DXVK_ASYNC=1 for dxvk-async builds automatically
    if let Ok(Some(dxvk)) = &config.try_get_selected_dxvk_info() {
        if dxvk.version.contains("async") {
            command.env("DXVK_ASYNC", "1");
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
        .map(|(key, value)| format!("{:?}=\"{:?}\"", key, value.unwrap_or_default()))
        .fold(String::new(), |acc, env| acc + " " + &env);

    tracing::info!("Running the game with command: {variables} bash -c \"{bash_chain}\"");

    command.current_dir(config.game.path).spawn()?;

    #[cfg(feature = "discord-rpc")]
    if config.launcher.discord_rpc.enabled {
        let rpc = DiscordRpc::new(config.launcher.discord_rpc);

        rpc.update(RpcUpdates::Connect)?;

        #[allow(unused_must_use)]
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));

            while let Ok(output) = Command::new("ps").arg("-A").stdout(Stdio::piped()).output() {
                let output = String::from_utf8_lossy(&output.stdout);

                if !output.contains("GenshinImpact.e") && !output.contains("unlocker.exe") {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_secs(3));
            }

            rpc.update(RpcUpdates::Disconnect);
        });
    }

    Ok(())
}
