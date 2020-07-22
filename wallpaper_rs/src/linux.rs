use super::Desktop;
use std::error::Error;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;

/// A desktop environment
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DesktopEnvt {
    GNOME,
    Cinnamon,
    MATE,
    XFCE,
    Deepin,
    KDE,
    BSPWM,
}

impl Desktop for DesktopEnvt {
    fn new() -> Result<Self, Box<dyn Error>> {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")?;
        if is_gnome_compliant(&desktop) {
            Ok(DesktopEnvt::GNOME)
        } else {
            Ok(match &desktop[..] {
                "X-Cinnamon" => DesktopEnvt::Cinnamon,
                "MATE" => DesktopEnvt::MATE,
                "XFCE" => DesktopEnvt::XFCE,
                "Deepin" => DesktopEnvt::Deepin,
                "KDE" => DesktopEnvt::KDE,
                "bspwm" => DesktopEnvt::BSPWM,
                _ => panic!("Unsupported Desktop Environment"),
            })
        }
    }

    fn set_wallpaper(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let path = enquote::enquote('"', &format!("{}", path));

        match self {
            DesktopEnvt::GNOME => {
                Command::new("gsettings")
                    .args(&["set", "org.gnome.desktop.background", "picture-uri", &path])
                    .output()?;
            }

            DesktopEnvt::Cinnamon => {
                Command::new("dconf")
                    .args(&[
                        "write",
                        "/org/cinnamon/desktop/background/picture-uri",
                        &path,
                    ])
                    .output()?;
            }

            DesktopEnvt::MATE => {
                let path_unquoted = enquote::unquote(&path).unwrap();
                let mate_path = path_unquoted
                    .strip_prefix("file://")
                    .unwrap();

                Command::new("dconf")
                    .args(&[
                        "write",
                        "/org/mate/desktop/background/picture-filename",
                        &mate_path,
                    ])
                    .output()?;
            }

            DesktopEnvt::XFCE => {
                let path_unquoted = enquote::unquote(&path).unwrap();
                let xfce_path = path_unquoted
                    .strip_prefix("file://")
                    .unwrap();
                
                // Get the raw output of xfconf-query for the wallpaper
                let values_raw = Command::new("xfconf-query")
                    .args(&[
                        "-c",
                        "xfce4-desktop",
                        "-p",
                        "/backdrop/screen0",
                        "-lv",
                    ])
                    .output()
                    .unwrap()
                    .stdout;

                // Filter out unwanted values (everything except */last-image)
                let values_str = match std::str::from_utf8(&values_raw) {
                    Ok(v) => v.to_string(),
                    Err(_) => "/backdrop/screen0/monitor0/workspace0/last-image".to_string(),
                };

                // Collect the keys for the filtered values
                let values_vec: Vec<&str> = values_str
                    .split_whitespace()
                    .step_by(2)
                    .filter(|v| v.contains("last-image"))
                    .collect();

                // Set all the keys to the new wallpaper
                for v in values_vec {
                    Command::new("xfconf-query")
                        .args(&[
                            "-c",
                            "xfce4-desktop",
                            "-p",
                            v,
                            "-s",
                            &xfce_path,
                        ])
                        .output()?;
                }
            }

            DesktopEnvt::Deepin => {
                Command::new("dconf")
                    .args(&[
                        "write",
                        "/com/deepin/wrap/gnome/desktop/background/picture-uri",
                        &path,
                    ])
                    .output()?;
            }

            DesktopEnvt::KDE => {
                // KDE needs plasma shell scripting to change the wallpaper
                let kde_set_arg = format!(
                    r#"
                    const monitors = desktops()
                    for (var i = 0; i < monitors.length; i++) {{
                        monitors[i].wallpaperPlugin = "org.kde.image"
                        monitors[i].currentConfigGroup = ["Wallpaper"]
                        monitors[i].writeConfig("Image", {})
                    }}"#,
                    &path
                );

                Command::new("qdbus")
                    .args(&[
                        "org.kde.plasmashell",
                        "/PlasmaShell",
                        "org.kde.PlasmaShell.evaluateScript",
                        &kde_set_arg,
                    ])
                    .output()?;
            }

            DesktopEnvt::BSPWM => {
                Command::new("feh")
                    .args(&["--bg-fill", &path.replace("\"", "")])
                    .output()?;
            }
        }

        Ok(())
    }

    fn get_wallpaper(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output = match self {
            DesktopEnvt::GNOME => Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.background", "picture-uri"])
                .output()?,

            DesktopEnvt::Cinnamon => Command::new("dconf")
                .arg("read")
                .arg("/org/cinnamon/desktop/background/picture-uri")
                .output()?,

            DesktopEnvt::MATE => Command::new("dconf")
                .args(&["read", "/org/mate/desktop/background/picture-filename"])
                .output()?,

            DesktopEnvt::XFCE => Command::new("xfconf-query")
                .args(&[
                    "-c",
                    "xfce4-desktop",
                    "-p",
                    "/backdrop/screen0/monitor0/workspace0/last-image",
                ])
                .output()?,

            DesktopEnvt::Deepin => Command::new("dconf")
                .args(&[
                    "read",
                    "/com/deepin/wrap/gnome/desktop/background/picture-uri",
                ])
                .output()?,
            DesktopEnvt::KDE => return Ok(kde_get_wallpaper()?),
            DesktopEnvt::BSPWM => Command::new("sed")
                .args(&[
                    "-n",
                    "'s/feh.*\\('.*'\\)/\\1/gp'",
                    &format!("/home/{}/.fehbg", std::env::var("USER")?.trim()),
                ])
                .output()?,
        };

        let output = enquote::unquote(String::from_utf8(output.stdout)?.trim().into())?;
        Ok(PathBuf::from(output))
    }
}

/// Check if desktop is Gnome compliant
fn is_gnome_compliant(desktop: &str) -> bool {
    desktop.contains("GNOME") || desktop == "Unity" || desktop == "Pantheon"
}

/// Returns the absolute wallpaper path on KDE, if possible.
///
/// It reads the first line starting with "Image="
/// in the file "~/.config/plasma-org.kde.plasma.desktop-appletsrc"
fn kde_get_wallpaper() -> Result<PathBuf, Box<dyn Error>> {
    let mut path = dirs_next::config_dir().ok_or("Could not determine config directory")?;
    path.push("plasma-org.kde.plasma.desktop-appletsrc");

    // Opening the file into a buffer reader
    let file = std::fs::File::open(path)?;

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("Image=") {
            let mut line = line[6..].trim();
            if line.starts_with("file://") {
                line = &line[7..];
            }
            return Ok(PathBuf::from(line));
        }
    }

    Err("KDE Image not found".into())
}