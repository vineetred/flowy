use std::process::Command;
use enquote;
use std::error::Error;

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