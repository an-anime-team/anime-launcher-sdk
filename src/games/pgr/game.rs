use std::process::{Command, Stdio};
use std::path::PathBuf;

use anime_game_core::pgr::telemetry;

use crate::components::wine::Bundle as WineBundle;

use crate::config::ConfigExt;
use crate::pgr::config::Config;

use crate::config::schema_blanks::prelude::{
    WineDrives,
    AllowedDrives
};

use crate::pgr::consts;

#[cfg(feature = "discord-rpc")]
use crate::discord_rpc::*;

#[cfg(feature = "sessions")]
use crate::{
    sessions::SessionsExt,
    pgr::sessions::Sessions
};

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

    if !config.game.path.exists() {
        return Err(anyhow::anyhow!("Game is not installed"));
    }

    let Some(wine) = config.get_selected_wine()? else {
        anyhow::bail!("Couldn't find wine executable");
    };

    let features = wine.features(&config.components.path)?.unwrap_or_default();

    let mut folders = Folders {
        wine: config.game.wine.builds.join(&wine.name),
        prefix: config.game.wine.prefix.clone(),
        game: config.game.path.clone(),
        temp: config.launcher.temp.clone().unwrap_or(std::env::temp_dir())
    };

    // Check telemetry servers

    tracing::info!("Checking telemetry");

    if let Ok(Some(server)) = telemetry::is_disabled() {
        return Err(anyhow::anyhow!("Telemetry server is not disabled: {server}"));
    }

    // Prepare wine prefix drives
    let prefix_folder = config.get_wine_prefix_path();

    config.game.wine.drives.map_folders(&folders.game, &prefix_folder)?;

    // Workaround for sandboxing feature
    if config.sandbox.enabled {
        WineDrives::map_folder(&prefix_folder, AllowedDrives::C, "../drive_c")?;
        WineDrives::map_folder(&prefix_folder, AllowedDrives::Z, "/")?;
    }

    // Prepare bash -c '<command>'
    // %command% = %bash_command% %windows_command% %launch_args%

    let mut bash_command = String::new();
    let mut windows_command = String::new();
    let mut launch_args = String::new();

    if config.game.enhancements.gamemode {
        bash_command += "gamemoderun ";
    }

    let run_command = features.command
        .map(|command| replace_keywords(command, &folders))
        .unwrap_or(format!("'{}'", folders.wine.join(wine.files.wine64.unwrap_or(wine.files.wine)).to_string_lossy()));

    bash_command += &run_command;
    bash_command += " ";

    if let Some(virtual_desktop) = config.game.wine.virtual_desktop.get_command("pgr") {
        windows_command += &virtual_desktop;
        windows_command += " ";
    }

    windows_command += "PGR.exe ";

    if config.game.wine.borderless {
        launch_args += "-screen-fullscreen 0 -popupwindow ";
    }

    // https://notabug.org/Krock/dawn/src/master/TWEAKS.md
    if config.game.enhancements.fsr.enabled {
        launch_args += "-window-mode exclusive ";
    }

    // gamescope <params> -- <command to run>
    if let Some(gamescope) = config.game.enhancements.gamescope.get_command() {
        bash_command = format!("{gamescope} -- {bash_command}");
    }

    // Bundle all windows arguments used to run the game into a single file
    if features.compact_launch {
        std::fs::write(folders.game.join("compact_launch.bat"), format!("start {windows_command} {launch_args}\nexit"))?;

        windows_command = String::from("compact_launch.bat");
        launch_args = String::new();
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

        windows_command = windows_command
            .replace(folders.wine.to_str().unwrap(), sandboxed_folders.wine.to_str().unwrap())
            .replace(folders.prefix.to_str().unwrap(), sandboxed_folders.prefix.to_str().unwrap())
            .replace(folders.game.to_str().unwrap(), sandboxed_folders.game.to_str().unwrap())
            .replace(folders.temp.to_str().unwrap(), sandboxed_folders.temp.to_str().unwrap());

        bash_command = format!("{bwrap} --chdir /tmp/sandbox/game -- {bash_command}");
        folders = sandboxed_folders;
    }

    // Finalize launching command
    bash_command = match &config.game.command {
        // Use user-given launch command
        Some(command) => replace_keywords(command, &folders)
            .replace("%command%", &format!("{bash_command} {windows_command} {launch_args}"))
            .replace("%bash_command%", &bash_command)
            .replace("%windows_command%", &windows_command)
            .replace("%launch_args%", &launch_args),

        // Combine bash and windows parts of the command
        None => format!("{bash_command} {windows_command} {launch_args}")
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

    let mut wine_folder = folders.wine.clone();

    if features.bundle == Some(WineBundle::Proton) {
        wine_folder.push("files");
    }

    command.envs(config.game.enhancements.hud.get_env_vars(config.game.enhancements.gamescope.enabled));
    command.envs(config.game.enhancements.fsr.get_env_vars());

    command.envs(config.game.wine.sync.get_env_vars());
    command.envs(config.game.wine.language.get_env_vars());
    command.envs(config.game.wine.shared_libraries.get_env_vars(wine_folder));

    command.envs(&config.game.environment);

    #[cfg(feature = "sessions")]
    if let Some(current) = Sessions::get_current()? {
        Sessions::apply(current, config.get_wine_prefix_path())?;
    }

    // Start Discord RPC just before the game
    #[cfg(feature = "discord-rpc")]
    let rpc = if config.launcher.discord_rpc.enabled {
        Some(DiscordRpc::new(config.launcher.discord_rpc.clone().into()))
    } else {
        None
    };

    #[cfg(feature = "discord-rpc")]
    if let Some(rpc) = &rpc {
        rpc.update(RpcUpdates::Connect)?;
    }

    // Run command

    let variables = command
        .get_envs()
        .map(|(key, value)| format!("{}=\"{}\"", key.to_string_lossy(), value.unwrap_or_default().to_string_lossy()))
        .fold(String::new(), |acc, env| acc + " " + &env);

    tracing::info!("Running the game with command: {variables} bash -c \"{bash_command}\"");

    // We use real current dir here because sandboxed one
    // obviously doesn't exist
    let mut child = command.current_dir(config.game.path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Create new game.log file to log all the game output
    let game_output = Arc::new(Mutex::new(
        File::create(consts::launcher_dir()?.join("game.log"))?
    ));

    let written = Arc::new(AtomicUsize::new(0));

    let mut stdout_join = None;
    let mut stderr_join = None;

    // Redirect stdout to the game.log file
    if let Some(mut stdout) = child.stdout.take() {
        let game_output = game_output.clone();
        let written = written.clone();

        stdout_join = Some(std::thread::spawn(move || -> std::io::Result<()> {
            let mut buf = [0; 1024];

            while let Ok(read) = stdout.read(&mut buf) {
                if read == 0 {
                    break;
                }

                let Ok(mut game_output) = game_output.lock() else {
                    break;
                };

                for line in buf[..read].split(|c| c == &b'\n') {
                    game_output.write_all(b"    [stdout] ")?;
                    game_output.write_all(line)?;
                    game_output.write_all(b"\n")?;

                    written.fetch_add(line.len() + 14, Ordering::Relaxed);
                }

                if written.load(Ordering::Relaxed) > *consts::GAME_LOG_FILE_LIMIT {
                    break;
                }
            }

            Ok(())
        }));
    }

    // Redirect stderr to the game.log file
    if let Some(mut stderr) = child.stderr.take() {
        let game_output = game_output.clone();
        let written = written.clone();

        stderr_join = Some(std::thread::spawn(move || -> std::io::Result<()> {
            let mut buf = [0; 1024];

            while let Ok(read) = stderr.read(&mut buf) {
                if read == 0 {
                    break;
                }

                let Ok(mut game_output) = game_output.lock() else {
                    break;
                };

                for line in buf[..read].split(|c| c == &b'\n') {
                    game_output.write_all(b"[!] [stderr] ")?;
                    game_output.write_all(line)?;
                    game_output.write_all(b"\n")?;

                    written.fetch_add(line.len() + 14, Ordering::Relaxed);
                }

                if written.load(Ordering::Relaxed) > *consts::GAME_LOG_FILE_LIMIT {
                    break;
                }
            }

            Ok(())
        }));
    }

    // Update discord RPC until the game process is closed
    while child.try_wait()?.is_none() {
        std::thread::sleep(std::time::Duration::from_secs(3));

        #[cfg(feature = "discord-rpc")]
        if let Some(rpc) = &rpc {
            rpc.update(RpcUpdates::Update)?;
        }
    }

    // Flush and close the game log file
    if let Ok(mut file) = game_output.lock() {
        file.flush()?;
    }

    drop(game_output);

    if let Some(join) = stdout_join {
        join.join().map_err(|err| anyhow::anyhow!("Failed to join stdout reader thread: {err:?}"))??;
    }

    if let Some(join) = stderr_join {
        join.join().map_err(|err| anyhow::anyhow!("Failed to join stderr reader thread: {err:?}"))??;
    }

    // Workaround for fast process closing (is it still a thing?)
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3));

        let output = Command::new("ps").arg("-A").stdout(Stdio::piped()).output()?;
        let output = String::from_utf8_lossy(&output.stdout);

        if !output.contains("PGR.exe") {
            break;
        }

        #[cfg(feature = "discord-rpc")]
        if let Some(rpc) = &rpc {
            rpc.update(RpcUpdates::Update)?;
        }
    }

    #[cfg(feature = "discord-rpc")]
    if let Some(rpc) = &rpc {
        rpc.update(RpcUpdates::Disconnect)?;
    }

    #[cfg(feature = "sessions")]
    if let Some(current) = Sessions::get_current()? {
        Sessions::update(current, config.get_wine_prefix_path())?;
    }

    Ok(())
}
