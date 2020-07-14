use wallpaper_rs::{Desktop, DesktopEnvt};

fn main() {
    let d = DesktopEnvt::new().expect("Desktop environment couldn't be determined");
    // This prints Ok(<file path>) on success and Err(<error message>) on failure
    println!("{:?}", d.get_wallpaper());
}
