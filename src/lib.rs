use std::process::Command;
use enquote;
use std::error::Error;
use clokwerk::{Scheduler, TimeUnits};
use std::thread;
use std::time::Duration;
use toml;
use serde::{Serialize,Deserialize};

/// Stores the times and filepaths as a vector of strings
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub times : Vec<String>,
    pub walls : Vec<String>,
}


/// args - NONE
/// return Result<String, Box<error>
/// Purpose - Get's path of the current wallpaper
pub fn get_wallpaper() -> Result<String,  Box<dyn Error>>{
    let op =   Command::new("dconf")
    .arg("read")
    .arg("/org/cinnamon/desktop/background/picture-uri")
    .output()?;

    return  Ok(enquote::unquote(String::from_utf8(op.stdout)?.trim().into())?)

    }

/// args - None
/// return <Result, Error>
/// Purpose - get the current envt
pub fn get_envt() -> Result<String, Box<dyn Error>> {

    Ok(std::env::var("XDG_CURRENT_DESKTOP")?)

}

/// args - filepath
/// return - Result<(), str>
/// purpose - set's the wallpaper to filepath
pub fn set_paper (path : &str) -> Result<(), Box<dyn Error>>  {

    let path = enquote::enquote('"', &format!("{}", path));
     Command::new("dconf")
        .args(&["write", "/org/cinnamon/desktop/background/picture-uri",&path])
        .output()?;
        Ok(())

}

// TODO - Someday, add some Result error return here
/// The main function that reads the config and runs the daemon
pub fn set_times () {
    let config = get_config("times.toml").unwrap();
    let walls = config.walls;
    let times = config.times;
    println!("Times - {:#?}", &times);
    println!("Paths - {:#?}", &walls);
    let mut scheduler = Scheduler::new();
    for (i, time) in times.into_iter().enumerate() {
        // Workaround becase Rust was being a bitch
        let wall = walls[i].clone();
        scheduler.every(1.day()).at(&time).run(move|| set_paper(&wall).unwrap());
    }
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Creates a new instance of struct Config and returns it
pub fn get_config(path : &str) -> Result<Config, Box<dyn Error>> {
    let toml_file = std::fs::read_to_string(path)?;
    let toml_data : Config = toml::from_str(&toml_file)?;
    
    Ok(toml_data)
}

/// Returns the contents of a given dir 
pub fn get_dir (path : &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files : Vec<String> = std::fs::read_dir(path)?.
    into_iter().
    map(|x| x.unwrap().path().display().to_string())
    .collect();

    // Appens file:// to the start of each item
    files = files
    .into_iter()
    .map(|y| "file://".to_string() + &y)
    .collect();
    // The read_dir iterator returns in an arbitrary manner
    // Sorted so that the images are viewed at the right time
    // Naming Mechanism - 00, 01, 02..
    files.sort();
    Ok(files)
}

/// Generates the config file. Takes the wallpaper folder path as args.
pub fn generate_config (path : &str) -> Result<(), Box<dyn Error>>{
    let files = get_dir(path)?;
    let length = files.len();
    let div = 1440/length;
    let mut times = Vec::new();
    let mut start_sec = 0;
    for _ in 0..length {
       times.push(format!("{}:{}",start_sec/60, start_sec%60 ));
       start_sec+=div;
    }

    let file = Config {
        times,
        walls : files,
    };

    let toml_string = toml::to_string(&file)?;
    std::fs::write("times.toml", toml_string)?;
    Ok(())
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_envt() {
        assert!(get_envt().is_ok());
    }


    #[test]
    fn test_get_wallpaper() {

        assert!(get_wallpaper().is_ok());
    }

    #[test]
    fn test_set_wallpaper() {
        // let t = get_envt();
        let  path = "file:///home/vineet/Desktop/69561.jpg";
        assert!(set_paper(path).is_ok());
    }
}
