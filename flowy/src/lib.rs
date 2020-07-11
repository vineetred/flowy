// THIS MODULE HANDLES GENERATION OF THE CONFIG FILE
// AND THE RUNNING OF THE DAEMON
use chrono::{DateTime, Local, Timelike};
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

pub fn generate_config_solar(path: &Path, lat: f64, long: f64) -> Result<(), Box<dyn Error>> {
    let mut day_walls = get_dir(path, "DAY")?;
    let night_walls = get_dir(path, "NIGHT")?;
    let unixtime = DateTime::timestamp(&chrono::offset::Utc::now()) as f64;

    let tt = solar::Timetable::new(unixtime, lat, long);
    let (sunrise, sunset) = tt.get_sunrise_sunset();
    println!("Sunrise {} Sunset {}", sunrise, sunset);
    let mut sunrise = solar::time_to_mins(sunrise);
    let mut sunset = solar::time_to_mins(sunset);

    let day_len = ((sunset as i32 - sunrise as i32) % 1440) as u32;
    let night_len = 1440 - day_len;
    let day_div = day_len / day_walls.len() as u32;
    let night_div = night_len / night_walls.len() as u32;
    let mut times = Vec::new();

    for _ in 0..day_walls.len() {
        times.push(format!("{}:{}", sunrise / 60, sunrise % 60));
        sunrise = (sunrise + day_div) % 1440;
    }

    for _ in 0..night_walls.len() {
        times.push(format!("{}:{}", sunset / 60, sunset % 60));
        sunset = (sunset + night_div) % 1440;
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
pub fn set_times(config: Config) {
    let walls = config.walls;
    let times = config.times;
    println!("Times - {:#?}", &times);
    println!("Paths - {:#?}", &walls);

    let desktop_envt = DesktopEnvt::new().expect("Desktop envt could not be determined");

    // set current wallpaper
    let current_index = get_current_wallpaper_idx(walls.len());
    desktop_envt.set_wallpaper(&walls[current_index]).unwrap();

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

/// Returns the index of the current wallpaper
/// depending on the number of wallpapers and the time of day.
fn get_current_wallpaper_idx(wall_len: usize) -> usize {
    const SECS_PER_DAY: u32 = 60 * 60 * 24;

    let time = Local::now().time();
    let time_relative = time.num_seconds_from_midnight() as f32 / SECS_PER_DAY as f32;
    let index = (wall_len as f32 * time_relative) as usize;
    // prevent overflow during leap seconds:
    index.min(wall_len - 1)
}
