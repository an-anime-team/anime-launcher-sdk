pub mod resolution;
pub mod repairer;
pub mod fsr;
pub mod hud;
pub mod fps;
pub mod window_mode;
pub mod dxvk;

pub mod wine;
pub mod gamescope;

pub mod prelude {
    pub use super::resolution::Resolution;
    pub use super::repairer::Repairer;
    pub use super::fsr::Fsr;
    pub use super::hud::HUD;
    pub use super::fps::Fps;
    pub use super::window_mode::WindowMode;

    pub use super::wine::prelude::*;
    pub use super::gamescope::prelude::*;
}
