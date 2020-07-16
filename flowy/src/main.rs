// CLI Import
use clap::{load_yaml, App};
use std::path::{Path, PathBuf};
mod presets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Housekeeping for Clap Arg parsing
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    // The times are set by themselves
    // Just supply the path and the TOML file is generated
    let dir = matches.value_of("dir");
    let preset = matches.value_of("preset");
    // Error checking for the Solar option
    if let Some(_) = matches.values_of("solar") {
        // Loading up the args into a vector
        let solar: Vec<_> = matches.values_of("solar").unwrap().collect();
        flowy::generate_config_solar(
            // Passing the Directory
            Path::new(solar[0]),
            // Passing the lat long
            solar[1].parse::<f64>().unwrap(),
            solar[2].parse::<f64>().unwrap(),
        )?;
    }
    // Since the functions are not required, this checks if
    // arguments have been passed to flowy
    // along with some error handling
    match flowy::match_dir(dir) {
        Ok(_) => (),
        Err(e) => eprintln!("Error with dir {}", e),
    }
    match presets::match_preset(preset) {
        Ok(_) => (),
        Err(e) => eprintln!("Error with preset {}", e),
    }
    // Runs forever
    let config = flowy::get_config()?;
    flowy::set_times(config)?;
    // Never reaches this but needed for Result return
    Ok(())
}
