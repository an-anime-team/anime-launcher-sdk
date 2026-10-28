use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Archive {
    path: PathBuf
}

impl Archive {
    pub fn open(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        if !path.exists() {
            anyhow::bail!("Archive does not exist: {path:?}");
        }

        Ok(Self {
            path
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn extract(&self, folder: impl AsRef<Path>) -> anyhow::Result<()> {
        let folder = folder.as_ref();

        std::fs::create_dir_all(folder)?;

        let filename = self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if filename.ends_with(".zip") && !filename.ends_with(".zip.001") {
            self.extract_zip(folder)
        }

        else if filename.ends_with(".7z")
            || filename.ends_with(".7z.001")
            || filename.ends_with(".zip.001")
            || filename.ends_with(".z01")
        {
            self.extract_7z(folder)
        }

        else {
            self.extract_tar(folder)
        }
    }

    fn extract_zip(&self, folder: &Path) -> anyhow::Result<()> {
        let status = Command::new("unzip")
            .arg("-q")
            .arg("-o")
            .arg(&self.path)
            .arg("-d")
            .arg(folder)
            .stdin(Stdio::null())
            .status();

        match status {
            Ok(status) if status.success() => Ok(()),

            _ => self.extract_7z(folder)
        }
    }

    fn extract_tar(&self, folder: &Path) -> anyhow::Result<()> {
        let status = Command::new("tar")
            .arg("-xf")
            .arg(&self.path)
            .arg("-C")
            .arg(folder)
            .stdin(Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract tar archive: {:?}", self.path);
        }

        Ok(())
    }

    fn extract_7z(&self, folder: &Path) -> anyhow::Result<()> {
        let status = Command::new(seven_zip()?)
            .arg("x")
            .arg(&self.path)
            .arg(format!("-o{}", folder.to_string_lossy()))
            .arg("-aoa")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract 7z archive: {:?}", self.path);
        }

        Ok(())
    }
}

fn seven_zip() -> anyhow::Result<&'static str> {
    for name in ["7z", "7za", "7zz"] {
        let status = Command::new(name)
            .arg("i")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if status.is_ok() {
            return Ok(name);
        }
    }

    anyhow::bail!("7z binary not found")
}
