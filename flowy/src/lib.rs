// THIS MODULE HANDLES GENERATION OF THE CONFIG FILE
// AND THE RUNNING OF THE DAEMON
use chrono::{DateTime, Local, Utc};
use clokwerk::{Scheduler, TimeUnits};
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
        files = files.into_iter().collect();
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
    let mut day_walls = get_dir(path, "DAY")?;
    let night_walls = get_dir(path, "NIGHT")?;
    let unixtime = DateTime::timestamp(&Utc::now()) as f64;

    let tt = solar::Timetable::new(unixtime, lat, long);
    let sunrise = tt.get(&solar::SolarTime::Sunrise).unwrap().round() as i64;
    let sunset = tt.get(&solar::SolarTime::Sunset).unwrap().round() as i64;

    // Day length in seconds
    let day_len = sunset - sunrise;
    // Night length in seconds
    let night_len = 86400 - day_len;
    // Offset in seconds for each wallpaper change during the day
    let day_div = day_len / day_walls.len() as i64;
    // Offset in seconds for each wallpaper change during the night
    let night_div = night_len / night_walls.len() as i64;
    let mut times = Vec::new();

    for i in 0..day_walls.len() {
        let absolute = sunrise + (i as i64) * day_div;
        let time_str: String = solar::unix_to_local(absolute).format("%H:%M").to_string();
        times.push(time_str);
    }

    for i in 0..night_walls.len() {
        let absolute = sunset + (i as i64) * night_div;
        let time_str: String = solar::unix_to_local(absolute).format("%H:%M").to_string();
        times.push(time_str);
    }

    day_walls.extend(night_walls);
    let config = Config {
        times,
        walls: day_walls,
    };
    let toml_string = toml::to_string(&config)?;
    std::fs::write(&get_config_path()?, toml_string)?;

    Ok(())
}

/// Generates the config file. Takes the wallpaper folder path as args.
pub fn generate_config(path: &Path) -> Result<(), Box<dyn Error>> {
    let walls = get_dir(path, "")?;
    let length = walls.len();
    let div = 1440 / length;
    let mut times = Vec::new();
    let mut start_sec = 0;
    for _ in 0..length {
        times.push(format!("{:02}:{:02}", start_sec / 60, start_sec % 60));
        start_sec += div;
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

// TODO - Someday, add some Result error return here
/// The main function that reads the config and runs the daemon
pub fn set_times(config: Config) -> Result<(), Box<dyn Error>> {
    let walls = config.walls;
    let times = config.times;
    println!("Times - {:#?}", &times);
    println!("Paths - {:#?}", &walls);

    let desktop_envt = DesktopEnvt::new().expect("Desktop envt could not be determined");

    // Set current wallpaper
    let current_index = get_current_wallpaper_idx(&times)?;
    desktop_envt.set_wallpaper(&walls[current_index])?;

    let mut scheduler = Scheduler::new();
    for (time, wall) in times.into_iter().zip(walls) {
        scheduler
            .every(1.day())
            .at(&time)
            .run(move || desktop_envt.set_wallpaper(&wall).unwrap());
    }
    loop {
        scheduler.run_pending();
        // Listens every minute
        thread::sleep(Duration::from_secs(60));
    }
}
/// Returns the index of the wallpaper path which is
/// closest to the current time
fn get_current_wallpaper_idx(wall_len: &Vec<String>) -> Result<usize, Box<dyn Error>> {
    // Get the current time
    let curr_time = Local::now().time();
    let mut global_min = 1440;
    let mut index = 0;
    for (i, time) in wall_len.into_iter().enumerate() {
        // Get the difference in absolute minutes
        let time = chrono::NaiveTime::parse_from_str(&time, "%H:%M")?;
        let min = curr_time.signed_duration_since(time).num_minutes().abs();
        // Compare and see if lowest we have seen so far
        if min < global_min {
            global_min = min;
            index = i;
        }
    }
    // Return the index of the lowest
    // time difference entry in Config.toml we saw
    Ok(index)
}
