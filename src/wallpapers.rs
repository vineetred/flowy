// THIS MODULE HANDLES THE SETTING AND GETTING
// OF THE WALLPAPER
use std::error::Error;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::DesktopEnvt;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::DesktopEnvt;

/// A trait implemented by desktop environments. It allows setting or getting a wallpaper.
pub trait Desktop: Sized {
    fn new() -> Result<Self, Box<dyn Error>>;

    fn set_wallpaper(&self, path: &str) -> Result<(), Box<dyn Error>>;

    fn get_wallpaper(&self) -> Result<PathBuf, Box<dyn Error>>;
}
