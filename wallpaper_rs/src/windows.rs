use super::Desktop;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
    SPI_SETDESKWALLPAPER,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DesktopEnvt;

impl Desktop for DesktopEnvt {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self)
    }

    fn set_wallpaper(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut path: Vec<u16> = OsStr::new(path).encode_wide().collect();
        // append null byte
        path.push(0);

        let successful = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                path.as_ptr() as *mut c_void,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            ) == 1
        };

        if successful {
            Ok(())
        } else {
            Err(io::Error::last_os_error().into())
        }
    }

    fn get_wallpaper(&self) -> Result<PathBuf, Box<dyn Error>> {
        let buffer: [u16; 260] = unsafe { std::mem::zeroed() };
        let successful = unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                buffer.len() as u32,
                buffer.as_ptr() as *mut c_void,
                0,
            ) == 1
        };

        if successful {
            // removes trailing zeroes from buffer
            let mut buffer = &buffer[..];
            while let Some(0) = buffer.last() {
                buffer = buffer.split_last().unwrap().1;
            }

            Ok(String::from_utf16(buffer)?.into())
        } else {
            Err(io::Error::last_os_error().into())
        }
    }
}
