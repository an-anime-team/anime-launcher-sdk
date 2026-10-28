pub mod archives;
pub mod downloader;
pub mod installer;

pub use self::archives::Archive;
pub use self::downloader::Downloader;
pub use self::installer::{Installer, Update};

pub mod prelude {
    pub use super::{
        Archive,
        Downloader,
        Installer,
        Update as InstallerUpdate
    };
}
