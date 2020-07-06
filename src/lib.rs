use clokwerk::{Scheduler, TimeUnits};
use enquote;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;
use toml;

pub fn match_dir(dir: Option<&str>) -> Result<(), Box<dyn Error>> {
    match dir {
        None => (),
        Some(dir) => match generate_config(dir) {
            Ok(_) => println!("Generated config file"),
            Err(e) => eprintln!("Error generating config file: {}", e),
        },
    }

    Ok(())
}
/// Stores the times and filepaths as a vector of strings
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub times: Vec<String>,
    pub walls: Vec<String>,
}
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

        // Panics since flowy does not support others yet
        _ => panic!("Unsupported Desktop Environment"),
    };

    return Ok(enquote::unquote(
        String::from_utf8(output.stdout)?.trim().into(),
    )?);
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
// TODO - Someday, add some Result error return here
/// The main function that reads the config and runs the daemon
pub fn set_times() {
    let config = get_config("times.toml").unwrap();
    let walls = config.walls;
    let times = config.times;
    println!("Times - {:#?}", &times);
    println!("Paths - {:#?}", &walls);
    let mut scheduler = Scheduler::new();
    for (i, time) in times.into_iter().enumerate() {
        // Workaround becase Rust was being a bitch
        let wall = walls[i].clone();
        scheduler
            .every(1.day())
            .at(&time)
            .run(move || set_paper(&wall).unwrap());
    }
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Creates a new instance of struct Config and returns it
pub fn get_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let toml_file = std::fs::read_to_string(path)?;
    let toml_data: Config = toml::from_str(&toml_file)?;

    Ok(toml_data)
}

/// Returns the contents of a given dir
pub fn get_dir(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files: Vec<String> = std::fs::read_dir(path)?
        .into_iter()
        .map(|x| x.unwrap().path().display().to_string())
        .collect();

    // Appens file:// to the start of each item
    if cfg!(target_os = "linux") {
        files = files
            .into_iter()
            .map(|y| "file://".to_string() + &y)
            .collect();
    }

    if cfg!(target_os = "macos") {
        files = files.into_iter().collect();
    }
    // The read_dir iterator returns in an arbitrary manner
    // Sorted so that the images are viewed at the right time
    // Naming Mechanism - 00, 01, 02..
    files.sort();
    Ok(files)
}

/// Generates the config file. Takes the wallpaper folder path as args.
pub fn generate_config(path: &str) -> Result<(), Box<dyn Error>> {
    let files = get_dir(path)?;
    let length = files.len();
    let div = 1440 / length;
    let mut times = Vec::new();
    let mut start_sec = 0;
    for _ in 0..length {
        times.push(format!("{}:{}", start_sec / 60, start_sec % 60));
        start_sec += div;
    }

    let file = Config {
        times,
        walls: files,
    };

    let toml_string = toml::to_string(&file)?;
    std::fs::write("times.toml", toml_string)?;
    Ok(())
}
