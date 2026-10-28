use std::path::PathBuf;

use super::archives::Archive;
use super::downloader::Downloader;

#[derive(Debug, Clone)]
pub enum Update {
    /// Downloading archive path.
    DownloadingStarted(PathBuf),

    /// `(current bytes, total bytes)`
    DownloadingProgress(u64, u64),

    DownloadingFinished,
    DownloadingError(String),

    /// Unpacking path.
    UnpackingStarted(PathBuf),

    UnpackingFinished,
    UnpackingError(String)
}

#[derive(Debug)]
pub struct Installer {
    pub downloader: Downloader,

    /// Path to the temp folder used to store archive before unpacking.
    pub temp_folder: PathBuf,

    /// Name of the archive file in the temp folder.
    pub filename: Option<String>
}

impl Installer {
    pub fn new(uri: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            downloader: Downloader::new(uri)?,
            temp_folder: std::env::temp_dir(),
            filename: None
        })
    }

    pub fn get_filename(&self) -> String {
        self.filename
            .clone()
            .unwrap_or_else(|| self.downloader.get_filename())
    }

    fn get_temp_path(&self) -> PathBuf {
        self.temp_folder.join(self.get_filename())
    }

    pub fn with_temp_folder(mut self, path: impl Into<PathBuf>) -> Self {
        self.temp_folder = path.into();

        self
    }

    pub fn with_filename(mut self, filename: impl ToString) -> Self {
        self.filename = Some(filename.to_string());

        self
    }

    /// Download the archive and unpack it into the given folder.
    pub async fn install(
        &self,
        unpack_to: impl Into<PathBuf>,
        updater: impl Fn(Update) + Send + Sync + 'static
    ) -> anyhow::Result<()> {
        let temp_path = self.get_temp_path();
        let unpack_to = unpack_to.into();

        updater(Update::DownloadingStarted(temp_path.clone()));

        if let Err(err) = self.downloader.download(&temp_path, |current, total| {
            updater(Update::DownloadingProgress(current, total));
        }).await {
            updater(Update::DownloadingError(err.to_string()));

            return Err(err);
        }

        updater(Update::DownloadingFinished);

        updater(Update::UnpackingStarted(unpack_to.clone()));

        if let Err(err) = Archive::open(&temp_path)
            .and_then(|archive| archive.extract(&unpack_to))
        {
            updater(Update::UnpackingError(err.to_string()));

            return Err(err);
        }

        updater(Update::UnpackingFinished);

        Ok(())
    }
}
