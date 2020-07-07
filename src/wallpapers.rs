// THIS MODULE HANDLES THE SETTING AND GETTING
// OF THE WALLPAPER
use std::error::Error;
use std::io::BufRead;
use std::process::Command;

/// Check if desktop is Gnome compliant
#[cfg(target_os = "linux")]
fn is_gnome_compliant(desktop: &str) -> bool {
    desktop.contains("GNOME") || desktop == "Unity" || desktop == "Pantheon"
}

/// args - NONE
/// return Result<String, Box<error>
/// Purpose - Get's path of the current wallpaper
#[cfg(target_os = "linux")]
pub fn get_wallpaper() -> Result<String, Box<dyn Error>> {
    let desktop = get_envt()?;

    let output = match desktop.as_str() {
        "GNOME" => Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.background", "picture-uri"])
            .output()?,

        "X-Cinnamon" => Command::new("dconf")
            .arg("read")
            .arg("/org/cinnamon/desktop/background/picture-uri")
            .output()?,

        "MATE" => Command::new("dconf")
            .args(&["read", "/org/mate/desktop/background/picture-filename"])
            .output()?,

        "XFCE" => Command::new("xfconf-query")
            .args(&[
                "-c",
                "xfce4-desktop",
                "-p",
                "/backdrop/screen0/monitor0/workspace0/last-image",
            ])
            .output()?,

        "Deepin" => Command::new("dconf")
            .args(&[
                "read",
                "/com/deepin/wrap/gnome/desktop/background/picture-uri",
            ])
            .output()?,

        "KDE" => return Ok(kde_get()?),

        // Panics since flowy does not support others yet
        _ => panic!("Unsupported Desktop Environment"),
    };

    Ok(enquote::unquote(
        String::from_utf8(output.stdout)?.trim().into(),
    )?)
}

#[cfg(target_os = "linux")]
fn kde_get() -> Result<String, Box<dyn Error>> {
    // Getting current directory and
    // appending the kde wallpaper
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

#[cfg(target_os = "macos")]
pub fn get_wallpaper() -> Result<String, Box<dyn Error>> {
    // Generate the Applescript string
    let cmd = r#"tell app "finder" to get posix path of (get desktop picture as alias)"#;
    // Run it using osascript
    let output = Command::new("osascript").args(&["-e", cmd]).output()?;

    Ok(String::from_utf8(output.stdout)?.trim().into())
}
/// args - None
/// return <Result, Error>
/// Purpose - get the current envt
#[cfg(target_os = "linux")]
pub fn get_envt() -> Result<String, Box<dyn Error>> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")?;
    if !is_gnome_compliant(&desktop) {
        return Ok(desktop);
    }
    Ok(String::from("GNOME"))
}

/// args - filepath
/// return - Result<(), Error>
/// purpose - set's the wallpaper to filepath
#[cfg(target_os = "linux")]
pub fn set_paper(path: &str) -> Result<(), Box<dyn Error>> {
    let path = enquote::enquote('"', &format!("{}", path));
    // Getting desktop here
    let desktop = get_envt()?;

    match desktop.as_str() {
        "GNOME" => {
            Command::new("gsettings")
                .args(&["set", "org.gnome.desktop.background", "picture-uri", &path])
                .output()?;
        }

        "X-Cinnamon" => {
            Command::new("dconf")
                .args(&[
                    "write",
                    "/org/cinnamon/desktop/background/picture-uri",
                    &path,
                ])
                .output()?;
        }

        "MATE" => {
            let mate_path = &path[7..];
            Command::new("dconf")
                .args(&[
                    "write",
                    "/org/mate/desktop/background/picture-filename",
                    &mate_path,
                ])
                .output()?;
        }

        "XFCE" => {
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

        "Deepin" => {
            Command::new("dconf")
                .args(&[
                    "write",
                    "/com/deepin/wrap/gnome/desktop/background/picture-uri",
                    &path,
                ])
                .output()?;
        }

        "KDE" => {
            // KDE needs plasma shell scripting
            // to change the wallpaper
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
        // Panics since flowy does not support others yet
        _ => panic!("Unsupported Desktop Environment"),
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_paper(path: &str) -> Result<(), Box<dyn Error>> {
    // Generate the Applescript string
    let cmd = &format!(
        r#"tell app "finder" to set desktop picture to POSIX file {}"#,
        enquote::enquote('"', path),
    );
    // Run it using osascript
    Command::new("osascript").args(&["-e", cmd]).output()?;

    Ok(())
}
