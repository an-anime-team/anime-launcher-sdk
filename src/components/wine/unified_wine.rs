use std::path::PathBuf;
use std::process::{Output, Child};
use std::ffi::OsStr;

use wincompatlib::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifiedWine {
    Default(Wine),
    Proton(Proton)
}

impl From<Wine> for UnifiedWine {
    #[inline]
    fn from(wine: Wine) -> Self {
        Self::Default(wine)
    }
}

impl From<Proton> for UnifiedWine {
    #[inline]
    fn from(proton: Proton) -> Self {
        Self::Proton(proton)
    }
}

impl From<UnifiedWine> for Wine {
    fn from(wine: UnifiedWine) -> Self {
        match wine {
            UnifiedWine::Default(wine) => wine,

            // Kind of unsafe but who cares?
            // I don't like proton anyway lol
            UnifiedWine::Proton(proton) => proton.wine().to_owned()
        }
    }
}

impl WineWithExt for UnifiedWine {
    #[inline]
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_prefix(prefix)),
            Self::Proton(proton) => Self::Proton(proton.with_prefix(prefix))
        }
    }

    #[inline]
    fn with_arch(self, arch: WineArch) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_arch(arch)),
            Self::Proton(proton) => Self::Proton(proton.with_arch(arch))
        }
    }

    #[inline]
    fn with_boot(self, boot: WineBoot) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_boot(boot)),
            Self::Proton(proton) => Self::Proton(proton.with_boot(boot))
        }
    }

    #[inline]
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_server(server)),
            Self::Proton(proton) => Self::Proton(proton.with_server(server))
        }
    }

    #[inline]
    fn with_loader(self, loader: WineLoader) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_loader(loader)),
            Self::Proton(proton) => Self::Proton(proton.with_loader(loader))
        }
    }

    #[inline]
    fn with_wine_libs(self, wine_libs: WineSharedLibs) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_wine_libs(wine_libs)),
            Self::Proton(proton) => Self::Proton(proton.with_wine_libs(wine_libs))
        }
    }

    #[inline]
    fn with_gstreamer_libs(self, gstreamer_libs: GstreamerSharedLibs) -> Self {
        match self {
            Self::Default(wine) => Self::Default(wine.with_gstreamer_libs(gstreamer_libs)),
            Self::Proton(proton) => Self::Proton(proton.with_gstreamer_libs(gstreamer_libs))
        }
    }
}

impl WineBootExt for UnifiedWine {
    #[inline]
    fn wineboot_command(&self) -> std::process::Command {
        match self {
            Self::Default(wine) => wine.wineboot_command(),
            Self::Proton(proton) => proton.wineboot_command()
        }
    }

    #[inline]
    fn init_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.init_prefix(path),
            Self::Proton(proton) => proton.init_prefix(path)
        }
    }

    #[inline]
    fn update_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.update_prefix(path),
            Self::Proton(proton) => proton.update_prefix(path)
        }
    }

    #[inline]
    fn stop_processes(&self, force: bool) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.stop_processes(force),
            Self::Proton(proton) => proton.stop_processes(force)
        }
    }

    #[inline]
    fn restart(&self) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.restart(),
            Self::Proton(proton) => proton.restart()
        }
    }

    #[inline]
    fn shutdown(&self) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.shutdown(),
            Self::Proton(proton) => proton.shutdown()
        }
    }

    #[inline]
    fn end_session(&self) -> anyhow::Result<Output> {
        match self {
            Self::Default(wine) => wine.end_session(),
            Self::Proton(proton) => proton.end_session()
        }
    }
}

impl WineRunExt for UnifiedWine {
    #[inline]
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> anyhow::Result<Child> {
        match self {
            Self::Default(wine) => wine.run(binary),
            Self::Proton(proton) => proton.run(binary)
        }
    }

    #[inline]
    fn run_args<T, S>(&self, args: T) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        match self {
            Self::Default(wine) => wine.run_args(args),
            Self::Proton(proton) => proton.run_args(args)
        }
    }

    #[inline]
    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        match self {
            Self::Default(wine) => wine.run_args_with_env(args, envs),
            Self::Proton(proton) => proton.run_args_with_env(args, envs)
        }
    }

    #[inline]
    fn winepath(&self, path: &str) -> anyhow::Result<PathBuf> {
        match self {
            Self::Default(wine) => wine.winepath(path),
            Self::Proton(proton) => proton.winepath(path)
        }
    }
}

impl WineOverridesExt for UnifiedWine {
    #[inline]
    fn add_override(&self, dll_name: impl AsRef<str>, modes: impl IntoIterator<Item = OverrideMode>) -> anyhow::Result<()> {
        match self {
            Self::Default(wine) => wine.add_override(dll_name, modes),
            Self::Proton(proton) => proton.add_override(dll_name, modes)
        }
    }

    #[inline]
    fn delete_override(&self, dll_name: impl AsRef<str>) -> anyhow::Result<()> {
        match self {
            Self::Default(wine) => wine.delete_override(dll_name),
            Self::Proton(proton) => proton.delete_override(dll_name)
        }
    }
}

impl WineFontsExt for UnifiedWine {
    #[inline]
    fn register_font(&self, ttf: impl AsRef<str>, font_name: impl AsRef<str>) -> anyhow::Result<()> {
        match self {
            Self::Default(wine) => wine.register_font(ttf, font_name),
            Self::Proton(proton) => proton.register_font(ttf, font_name)
        }
    }

    #[inline]
    fn font_is_installed(&self, ttf: impl AsRef<str>) -> bool {
        match self {
            Self::Default(wine) => wine.font_is_installed(ttf),
            Self::Proton(proton) => proton.font_is_installed(ttf)
        }
    }

    #[inline]
    fn install_corefont(&self, corefont: Corefont) -> anyhow::Result<()> {
        match self {
            Self::Default(wine) => wine.install_corefont(corefont),
            Self::Proton(proton) => proton.install_corefont(corefont)
        }
    }
}
