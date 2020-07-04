fn main() {
        // The times are set by themselves
        // Just supply the path and the TOML file is generated
        flowy::generate_config("/home/vineet/Downloads/walls").unwrap();
        // Runs forever
        flowy::set_times();

}