use std::path::Path;
use std::fs::File;
use std::io::Write;

use anime_game_core::sophon::reqwest;

#[derive(Debug)]
pub struct Downloader {
    client: reqwest::Client,
    uri: String
}

impl Downloader {
    pub fn new(uri: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()?,
            uri: uri.into()
        })
    }

    /// Get file name from the URI.
    ///
    /// - `https://example.com/example.zip` -> `example.zip`
    /// - `https://example.com` -> `index.html`
    pub fn get_filename(&self) -> String {
        self.uri
            .split('/')
            .next_back()
            .filter(|name| !name.is_empty())
            .map(|name| name.split('?').next().unwrap_or(name).to_string())
            .unwrap_or_else(|| String::from("index.html"))
    }

    /// Download the file to the given path, reporting progress in bytes.
    pub async fn download(
        &self,
        path: impl AsRef<Path>,
        progress: impl Fn(u64, u64) + Send
    ) -> anyhow::Result<()> {
        let path = path.as_ref();

        let mut response = self
            .client
            .get(&self.uri)
            .send()
            .await?
            .error_for_status()?;

        let total = response.content_length().unwrap_or(0);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        let mut downloaded = 0;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;

            downloaded += chunk.len() as u64;

            progress(downloaded, total);
        }

        Ok(())
    }
}
