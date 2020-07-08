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
///
/// On platforms where only one desktop environment exists (e.g. Windows, macOS), this can
/// be implemented with a zero-sized type. On Linux, it is an enum.
pub trait Desktop: Sized {
    /// Creates a new instance of this desktop.
    ///
    /// On Linux, this function detects the desktop environment.
    /// It panics if the desktop environment is unsupported. It returns an error
    /// if the desktop environment couldn't be determined (i.e., the `XDG_CURRENT_DESKTOP`
    /// environment variable isn't set).
    fn new() -> Result<Self, Box<dyn Error>>;

    /// Sets the wallpaper for all computer screens to the specified file path.
    ///
    /// The file should be an image file supported by the patform, e.g. a JPEG.
    fn set_wallpaper(&self, path: &str) -> Result<(), Box<dyn Error>>;

    /// Returns the file path to the image used as the wallpaper.
    ///
    /// If different screens have different wallpapers, only one of them is returned;
    /// the behavior depends on the platform and desktop environment.
    fn get_wallpaper(&self) -> Result<PathBuf, Box<dyn Error>>;
}
