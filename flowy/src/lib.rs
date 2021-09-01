// THIS MODULE HANDLES GENERATION OF THE CONFIG FILE
// AND THE RUNNING OF THE DAEMON
use chrono::{DateTime, Local, NaiveTime, Utc};
use directories_next::BaseDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use wallpaper_rs::{Desktop, DesktopEnvt};
mod solar;

/// Basic error handling to ensure
/// an empty args field does not
/// crash the app
pub fn match_dir(dir: Option<&str>) -> Result<(), Box<dyn Error>> {
    match dir {
        None => (),
        Some(dir) => match generate_config(Path::new(dir)) {
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

/// Creates a new instance of struct Config and returns it
pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_path = get_config_path()?;
    let toml_file = std::fs::read_to_string(&config_path)?;
    let toml_data: Config = toml::from_str(&toml_file)?;

    Ok(toml_data)
}

/// Returns the contents of a given dir
pub fn get_dir(path: &Path, solar_filter: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files: Vec<String> = std::fs::read_dir(path)?
        .into_iter()
        .map(|x| x.unwrap().path().display().to_string())
        .filter(|y| y.contains(solar_filter))
        .collect();

    // Appens file:// to the start of each item
    if cfg!(target_os = "linux") {
        files = files
            .into_iter()
            .map(|y| "file://".to_string() + &y)
            .filter(|y| y.contains(solar_filter))
            .collect();
    }

    if cfg!(target_os = "macos") {
        files = files.into_iter()
        .filter(|y| y.contains(solar_filter))
        .collect();
    }
    // The read_dir iterator returns in an arbitrary manner
    // Sorted so that the images are viewed at the right time
    // Naming Mechanism - 00, 01, 02..
    files.sort();
    Ok(files)
}

/// Does esentially the same thing as generate_config
/// Only runs when sunrise and sunset times
/// need to be accounted for
/// Takes lat and long of a location along with the wallpaper path
pub fn generate_config_solar(path: &Path, lat: f64, long: f64) -> Result<(), Box<dyn Error>> {
    println!("<---- Solar Mode ---->");
    println!("Lat: {} Long: {}", &lat, &long);
    // Checking for the night and day prefix
    let mut day_walls = get_dir(path, "DAY")?;
    let night_walls = get_dir(path, "NIGHT")?;
    let unixtime = DateTime::timestamp(&Utc::now()) as f64;
    // Creating solar table based on time, lat, long
    let tt = solar::Timetable::new(unixtime, lat, long);
    let (sunrise, sunset) = tt.get_sunrise_sunset();

    // Day length in seconds
    let day_len = (sunset - sunrise) % 86400;
    // Night length in seconds
    let night_len = (86400 - day_len) % 86400;
    // Offset in seconds for each wallpaper change during the day
    let day_div = day_len / (day_walls.len()) as i64;
    // Offset in seconds for each wallpaper change during the night
    let night_div = night_len / (night_walls.len()) as i64;
    let mut times = Vec::new();

    // Adding times and paths
    for i in 0..day_walls.len() {
        let absolute = sunrise + (day_div * (i as i64));
        let time_str: String = solar::unix_to_local(absolute).format("%H:%M").to_string();
        times.push(time_str);
    }

    for i in 0..night_walls.len() {
        let absolute = sunset + (night_div * (i as i64));
        let time_str: String = solar::unix_to_local(absolute).format("%H:%M").to_string();
        times.push(time_str);
    }
    // Loading all the night paths to day paths
    day_walls.extend(night_walls);
    let config = Config {
        times,
        walls: day_walls,
    };
    // Writing times and paths to config.toml
    let toml_string = toml::to_string(&config)?;
    std::fs::write(&get_config_path()?, toml_string)?;

    Ok(())
}

/// Generates the config file. Takes the wallpaper folder path as args.
pub fn generate_config(path: &Path) -> Result<(), Box<dyn Error>> {
    println!("<---- Normal Mode ---->");
    let walls = get_dir(path, "")?;
    // Offset in seconds for each wallpaper
    let div = 86400 / walls.len();
    let mut times = Vec::new();

    for i in 0..walls.len() {
        let offset = div * i;
        times.push(format!("{:02}:{:02}", offset / 3600, (offset / 60) % 60));
    }

    let config = Config { times, walls };

    let toml_string = toml::to_string(&config)?;
    std::fs::write(&get_config_path()?, toml_string)?;
    Ok(())
}

/// Returns the path of the config directory. If the directory doesn't exist, it is created.
pub fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
    let base_dirs = BaseDirs::new().expect("Couldn't get base directory for the config file");
    let mut config_file = base_dirs.config_dir().to_path_buf();
    config_file.push("flowy");
    std::fs::create_dir_all(&config_file)?;
    Ok(config_file)
}

/// Returns the path where the config file is stored
fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let mut config_file = get_config_dir()?;
    config_file.push("config.toml");
    Ok(config_file)
}

/// Parses the config file and runs the daemon
pub fn set_times(config: Config) -> Result<(), Box<dyn Error>> {
    let walls = config.walls;
    let times = config.times;
    println!("Wallpapers:");
    for i in 0..times.len() {
        println!("- {:?} = {:?}", times[i], &walls[i]);
    }
    // Will throw an error if Desktop Envt is not supported
    let desktop_envt = DesktopEnvt::new().expect("Desktop envt could not be determined");
    // Create an instance of last_index pointing to None
    let mut last_index = None;
    println!("<--- Daemon Listening --->");
    // This daemon checks every minute if the index of the wallpaper has changed
    // If yes, then the new wallpaper is 
    loop {
        // Getting the current wallpaper's index
        let current_index = get_current_wallpaper_idx(&times)?;
        if Some(current_index) != last_index {
            // Updating last_index to the current_index
            last_index = Some(current_index);
            // Set current wallpaper
            desktop_envt.set_wallpaper(&walls[current_index])?;
        }
        // Check every t seconds
        // Change this if you would like a more accurate daemon
        let t = 60;
        thread::sleep(Duration::from_secs(t));
    }
}

/// Returns the index of the wallpaper which should be displayed now.
///
/// For example, if the times are "00:00", "01:00" and "02:00", the first image
/// should be shown from 00:00 to 00:59 and the second image from 01:00 to 01:59.
///
/// Therefore, this function returns the index of the _last_ time that isn't
/// greater than the current time.
fn get_current_wallpaper_idx(wall_times: &[String]) -> Result<usize, Box<dyn Error>> {
    if wall_times.is_empty() {
        panic!("Array of times can't be empty");
    }

    // Get the current time
    let curr_time = Local::now().time();

    // Looping through times to compare all of them
    let mut index = 0;
    for time in wall_times {
        let time = NaiveTime::parse_from_str(&time, "%H:%M")?;
        if time > curr_time {
            break;
        }
        index += 1;
    }

    // In case the current time is lower that the first time,
    // we just loop back to the time before it
    if index == 0 {
        index = wall_times.len() - 1;
        return Ok(index);
    }
    Ok(index - 1)
}
