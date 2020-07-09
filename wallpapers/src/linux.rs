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
                let mate_path = &path[7..];
                Command::new("dconf")
                    .args(&[
                        "write",
                        "/org/mate/desktop/background/picture-filename",
                        &mate_path,
                    ])
                    .output()?;
            }

            DesktopEnvt::XFCE => {
                let xfce_path = &path[7..];
                Command::new("xfconf-query")
                    .args(&[
                        "-c",
                        "xfce4-desktop",
                        "-p",
                        "/backdrop/screen0/monitor0/workspace0/last-image",
                        "-s",
                        &xfce_path,
                    ])
                    .output()?;
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

            DesktopEnvt::KDE => return Ok(kde_get()?),
        };

        let output = enquote::unquote(String::from_utf8(output.stdout)?.trim().into())?;
        Ok(PathBuf::from(output))
    }
}

/// Check if desktop is Gnome compliant
fn is_gnome_compliant(desktop: &str) -> bool {
    desktop.contains("GNOME") || desktop == "Unity" || desktop == "Pantheon"
}

fn kde_get() -> Result<PathBuf, Box<dyn Error>> {
    // Getting current directory and
    // appending the KDE wallpaper
    // repo to the end of the path
    let mut path = std::env::current_dir()?.display().to_string();
    path.push_str("/plasma-org.kde.plasma.desktop-appletsrc");
    // Opening the file into a buffer reader
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("Image=") {
            return Ok(line[6..].trim().into());
        }
    }

    Err("KDE Image not found".into())
}
