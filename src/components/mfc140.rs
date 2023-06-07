use std::path::Path;
use std::process::Command;

use anime_game_core::installer::downloader::Downloader;

// Source: https://github.com/Winetricks/winetricks/blob/e9454179686b3659ad3f47a5d49e6e4e82862cd5/src/winetricks#L13206

const URL: &str = "https://download.microsoft.com/download/9/3/F/93FCF1E7-E6A4-478B-96E7-D4B285925B00/vc_redist.x64.exe";

const LIBRARIES: &[&str] = &[
    "mfc140.dll",
    "mfc140u.dll",
    "mfcm140.dll",
    "mfcm140u.dll"
];

pub fn is_installed(wine_prefix: impl AsRef<Path>) -> bool {
    wine_prefix.as_ref().join("drive_c/windows/system32/mfc140.dll").exists()
}

pub fn install(wine_prefix: impl AsRef<Path>) -> anyhow::Result<()> {
    let vcredist = std::env::temp_dir().join("vcredist/vc_redist.x86.exe");
    let vcredist_extracted = std::env::temp_dir().join("vcredist/extracted");

    Downloader::new(URL)?
        .with_continue_downloading(false)
        .download(&vcredist, |_, _| {})?;

    // w_try_cabextract --directory="${W_TMP}/win64"  "${W_CACHE}"/vcrun2015/vc_redist.x64.exe -F 'a11'
    let output = Command::new("cabextract")
        .arg("-d")
        .arg(&vcredist_extracted)
        .arg(vcredist)
        .arg("-F")
        .arg("a11")
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to extract vcrun2015 (1): {}", String::from_utf8_lossy(&output.stderr));
    }

    // w_try_cabextract --directory="${W_TMP}/win64" "${W_TMP}/win64/a11"
    let output = Command::new("cabextract")
        .arg("-d")
        .arg(&vcredist_extracted)
        .arg(vcredist_extracted.join("a11"))
        .arg("-F")
        .arg("a11")
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to extract vcrun2015 (2): {}", String::from_utf8_lossy(&output.stderr));
    }

    // w_try_cp_dll "${W_TMP}/win64"/mfc140.dll "${W_SYSTEM64_DLLS}"/mfc140.dll
    for lib in LIBRARIES {
        let dest = wine_prefix.as_ref().join("drive_c/windows/system32").join(lib);

        if dest.exists() {
            std::fs::remove_file(&dest)?;
        }

        std::fs::copy(vcredist_extracted.join(lib), dest)?;
    }

    Ok(())
}
