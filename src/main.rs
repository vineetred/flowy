// CLI Import
use clap::{load_yaml, App};
mod presets;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Housekeeping for Clap Arg parsing
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    // The times are set by themselves
    // Just supply the path and the TOML file is generated
    let dir = matches.value_of("dir");
    let preset = matches.value_of("preset");
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
    flowy::set_times();

    Ok(())
}
