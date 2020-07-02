use std::process::Command;
use enquote;

/// args - NONE
/// return Result<String, Box<error>
/// Purpose - Get's path of the current wallpaper
pub fn get_wallpaper() -> Result<String,  Box<dyn std::error::Error>>{
    let op =   Command::new("dconf")
    .arg("read")
    .arg("/org/cinnamon/desktop/background/picture-uri")
    .output()?;

    return  Ok(enquote::unquote(String::from_utf8(op.stdout)?.trim().into())?)

    }


/// args - None
/// return <Result, str>
/// Purpose - get the current envt
pub fn get_envt<'a>() -> Result<String, &'a str> {
    match std::env::var("XDG_CURRENT_DESKTOP") {
        Ok(desktop) => Ok(desktop),
        Err(_) => return Err("Could not find desktop"),

    }
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