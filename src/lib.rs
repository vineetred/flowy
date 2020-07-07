// THIS MODULE HANDLES GENERATION OF THE CONFIG FILE
// AND THE RUNNING OF THE DAEMON
use clokwerk::{Scheduler, TimeUnits};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::thread;
use std::time::Duration;
mod wallpapers;

/// Basic error handling to ensure
/// an empty args field does not
/// crash the app
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
        times.push(format!("{:02}:{:02}", start_sec / 60, start_sec % 60));
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

// TODO - Someday, add some Result error return here
/// The main function that reads the config and runs the daemon
pub fn set_times() {
    let config = get_config("times.toml").unwrap();
    let walls = config.walls;
    let times = config.times;
    println!("Times - {:#?}", &times);
    println!("Paths - {:#?}", &walls);
    let mut scheduler = Scheduler::new();
    for (time, wall) in times.into_iter().zip(walls) {
        scheduler
            .every(1.day())
            .at(&time)
            .run(move || wallpapers::set_paper(&wall).unwrap());
    }
    loop {
        scheduler.run_pending();
        // Listens every minute
        thread::sleep(Duration::from_secs(60));
    }
}
