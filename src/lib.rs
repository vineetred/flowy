use std::process::Command;
use enquote;
use std::error::Error;
use clokwerk::{Scheduler, TimeUnits};
use std::thread;
use std::time::Duration;
use toml;
use serde::Deserialize;

/// Stores the times as a vector of strings
#[derive(Debug, Deserialize)]
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
pub fn set_paper (path : &str) -> Result<(), &'static str>  {

    let path = enquote::enquote('"', &format!("{}", path));
    match Command::new("dconf")
        .args(&["write", "/org/cinnamon/desktop/background/picture-uri",&path])
        .output() {
            Ok(_) => Ok(()),
            Err(_) => Err("Error changing paper"),
        }


}

pub fn set_times () {
    
    let config = get_config("times.toml");
    let walls = config.walls;
    let times = config.times;
    let mut scheduler = Scheduler::new();
    for (i, time) in times.into_iter().enumerate() {
        // Workaround becase Rust was being a bitch
        let wall = walls[i].clone();
        scheduler.every(1.day()).at(&time).run(move|| set_paper(&wall).unwrap());
    }
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(10));
    }
}


pub fn get_config(path : &str) -> Config {
    let toml_file = std::fs::read_to_string(path).unwrap();
    let toml_data : Config = toml::from_str(&toml_file).unwrap();
    toml_data
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