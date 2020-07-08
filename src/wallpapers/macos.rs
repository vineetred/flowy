use super::Desktop;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DesktopEnvt;

impl Desktop for DesktopEnvt {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    fn set_wallpaper(&self, path: &str) -> Result<(), Box<dyn Error>> {
        // Generate the Applescript string
        let cmd = &format!(
            r#"tell app "finder" to set desktop picture to POSIX file {}"#,
            enquote::enquote('"', path),
        );
        // Run it using osascript
        Command::new("osascript").args(&["-e", cmd]).output()?;

        Ok(())
    }

    fn get_wallpaper(&self) -> Result<PathBuf, Box<dyn Error>> {
        // Generate the Applescript string
        let cmd = r#"tell app "finder" to get posix path of (get desktop picture as alias)"#;
        // Run it using osascript
        let output = Command::new("osascript").args(&["-e", cmd]).output()?;

        Ok(String::from_utf8(output.stdout)?.trim().into())
    }
}
