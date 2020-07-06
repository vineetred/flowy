// CLI Import
use clap::{load_yaml, App};
fn main() {
    // Housekeeping for Clap Arg parsing
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    // The times are set by themselves
    // Just supply the path and the TOML file is generated
    let dir = matches.value_of("dir").unwrap();

    match flowy::generate_config(dir) {
        Ok(_) => println!("Generated config file"),
        Err(e) => eprintln!("Error generating config file: {}", e),
    }
    // // Runs forever
    flowy::set_times();
}
