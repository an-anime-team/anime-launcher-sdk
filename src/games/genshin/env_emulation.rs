use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};

use serde::{Serialize, Deserialize};
use enum_ordinalize::Ordinalize;

use anime_game_core::reqwest::blocking::Client;
use anime_game_core::reqwest::header::{USER_AGENT, REFERER, RANGE};

use crate::genshin::consts::cache_dir;

/// Official Bilibili PC games SDK package containing the channel server plugin
pub const BILIBILI_PLUGIN_URI: &str = "https://pkg.biligame.com/games/PCGameSDK5.1.0/051985/PCGameSDK5.1.0.zip";

/// Name of the Bilibili SDK package cached in the launcher's cache folder
pub const BILIBILI_PLUGIN_ARCHIVE: &str = "PCGameSDK5.1.0.zip";

/// Folder in the China edition game directory the plugin should be installed to
pub const BILIBILI_PLUGIN_FOLDER: &str = "YuanShen_Data/Plugins";

/// SDK package CDN returns 403 Forbidden without a browser-like User-Agent
const BILIBILI_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

const BILIBILI_REFERER: &str = "https://open.biligame.com/";

/// Name of the 64 bit plugin entry inside the SDK package.
///
/// Entry names in the package are GBK-encoded (so `64位` is stored as
/// `64\xce\xbb`), but they also contain a Unicode path extra field with a
/// UTF-8 name which the `zip` crate prefers. Both representations start with
/// `64`, so the entry is matched by that prefix and doesn't depend on which
/// one of them is used.
const BILIBILI_PLUGIN_ENTRY_SUFFIX: &str = "/PCGameSDK.dll";

const BILIBILI_PLUGIN_ENTRY_ARCH_PREFIX: &str = "64";

/// Minimal amount of downloaded bytes between progress updates
const PROGRESS_UPDATE_STEP: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ordinalize)]
pub enum Environment {
    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=mihoyo
    /// game_version=[game version]
    /// sub_channel=0
    /// ```
    PC,

    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=pcseaepic
    /// game_version=[game version]
    /// # plugin_sdk_version=2.14.2 (??? not used now)
    /// sub_channel=3
    /// ```
    Epic,

    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=1
    /// cps=pcgoogle
    /// game_version=[game version]
    /// sub_channel=6
    /// ```
    Android,

    /// Bilibili channel server. Requires the China (`YuanShen`) edition of the
    /// game and the `YuanShen_Data/Plugins/PCGameSDK.dll` plugin installed.
    ///
    /// `config.ini` format:
    /// 
    /// ```ini
    /// [General]
    /// channel=14
    /// cps=bilibili
    /// game_version=[game version]
    /// sub_channel=0
    /// ```
    Bilibili
}

impl Default for Environment {
    #[inline]
    fn default() -> Self {
        Self::PC
    }
}

impl Environment {
    /// Generate `config.ini`'s content
    pub fn generate_config(&self, game_version: impl AsRef<str>) -> String {
        match self {
            Self::PC => [
                "[General]",
                "channel=1",
                "cps=mihoyo",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=0"
            ].join("\n"),

            Self::Epic => [
                "[General]",
                "channel=1",
                "cps=pcseaepic",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=3"
            ].join("\n"),

            Self::Android => [
                "[General]",
                "channel=1",
                "cps=pcgoogle",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=6"
            ].join("\n"),

            Self::Bilibili => [
                "[General]",
                "channel=14",
                "cps=bilibili",
                &format!("game_version={}", game_version.as_ref()),
                "sub_channel=0"
            ].join("\n")
        }
    }
}

/// Get path to the Bilibili channel server plugin in the given
/// (China edition) game folder
pub fn bilibili_plugin_path(game_path: impl AsRef<Path>) -> PathBuf {
    game_path.as_ref().join(BILIBILI_PLUGIN_FOLDER).join("PCGameSDK.dll")
}

/// Check whether the Bilibili channel server plugin is installed
///
/// The game silently falls back to the official (HoYoverse) server when this
/// plugin is missing, so the launcher verifies its presence before launching.
pub fn is_bilibili_plugin_installed(game_path: impl AsRef<Path>) -> bool {
    std::fs::metadata(bilibili_plugin_path(game_path))
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

/// Download the Bilibili plugin package (if it's not cached yet) and return
/// path to it
fn download_bilibili_plugin_archive<F: Fn(u64, u64)>(progress: F) -> anyhow::Result<PathBuf> {
    let folder = cache_dir()?.join("bilibili");
    let archive = folder.join(BILIBILI_PLUGIN_ARCHIVE);

    // The archive is only moved to this path after a successful download,
    // so its existence means it's complete
    if archive.is_file() {
        return Ok(archive);
    }

    tracing::info!("Downloading Bilibili plugin package");

    std::fs::create_dir_all(&folder)?;

    let part = folder.join(format!("{BILIBILI_PLUGIN_ARCHIVE}.part"));

    // Continue downloading from the previous attempt if possible
    let existing = std::fs::metadata(&part)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let mut request = client
        .get(BILIBILI_PLUGIN_URI)
        .header(USER_AGENT, BILIBILI_USER_AGENT)
        .header(REFERER, BILIBILI_REFERER);

    if existing > 0 {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }

    let mut response = request.send()?.error_for_status()?;

    // Server has to confirm it continues the file instead of sending it from
    // the very beginning
    let resumed = existing > 0 && response.status().as_u16() == 206;

    let downloaded = if resumed { existing } else { 0 };
    let total = downloaded + response.content_length().unwrap_or(0);

    let mut file = if resumed {
        OpenOptions::new().append(true).open(&part)?
    }

    else {
        File::create(&part)?
    };

    progress(downloaded, total);

    let mut buffer = [0; 1024 * 64];
    let mut downloaded = downloaded;
    let mut last_reported = downloaded;

    loop {
        let read = response.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        file.write_all(&buffer[..read])?;

        downloaded += read as u64;

        // Don't spam progress updates: on a fast connection the file is
        // downloaded in thousands of chunks
        if downloaded - last_reported >= PROGRESS_UPDATE_STEP {
            last_reported = downloaded;

            progress(downloaded, total);
        }
    }

    progress(downloaded, total);

    file.flush()?;

    drop(file);

    std::fs::rename(&part, &archive)?;

    Ok(archive)
}

/// Download official Bilibili PC games SDK package and install `PCGameSDK.dll`
/// from it into the given (China edition) game folder
///
/// Plugin is required for the game to actually connect to the Bilibili channel
/// server. Its absence makes the game silently fall back to the official server.
#[tracing::instrument(level = "debug", skip(progress))]
pub fn install_bilibili_plugin(
    game_path: impl AsRef<Path> + std::fmt::Debug,
    progress: impl Fn(u64, u64) + Send + 'static
) -> anyhow::Result<()> {
    let archive = download_bilibili_plugin_archive(&progress)?;

    tracing::info!("Installing Bilibili plugin");

    let destination = bilibili_plugin_path(game_path);

    let parent = destination.parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get Bilibili plugin folder"))?
        .to_path_buf();

    std::fs::create_dir_all(&parent)?;

    let mut zip = match zip::ZipArchive::new(File::open(&archive)?) {
        Ok(zip) => zip,

        Err(err) => {
            // Remove the cached archive so the next attempt will download it again
            let _ = std::fs::remove_file(&archive);

            return Err(err.into());
        }
    };

    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;

        let is_plugin = {
            let name = entry.name();

            name.ends_with(BILIBILI_PLUGIN_ENTRY_SUFFIX)
                && name
                    .split('/')
                    .any(|part| part.starts_with(BILIBILI_PLUGIN_ENTRY_ARCH_PREFIX))
        };

        if !is_plugin {
            continue;
        }

        // Write to a temporary file first to not leave a half-written library
        // in the game folder in case of an error
        let temp = parent.join("PCGameSDK.dll.part");

        let mut file = File::create(&temp)?;

        std::io::copy(&mut entry, &mut file)?;

        file.flush()?;

        drop(file);

        std::fs::rename(&temp, &destination)?;

        tracing::info!("Installed Bilibili plugin to {}", destination.to_string_lossy());

        return Ok(());
    }

    anyhow::bail!("PCGameSDK.dll is not found in the Bilibili SDK package")
}

#[cfg(test)]
mod tests {
    use super::Environment;

    #[test]
    fn generate_config() {
        assert_eq!(
            Environment::PC.generate_config("5.0.0"),
            "[General]\nchannel=1\ncps=mihoyo\ngame_version=5.0.0\nsub_channel=0"
        );

        assert_eq!(
            Environment::Android.generate_config("5.0.0"),
            "[General]\nchannel=1\ncps=pcgoogle\ngame_version=5.0.0\nsub_channel=6"
        );

        assert_eq!(
            Environment::Bilibili.generate_config("5.0.0"),
            "[General]\nchannel=14\ncps=bilibili\ngame_version=5.0.0\nsub_channel=0"
        );
    }
}
