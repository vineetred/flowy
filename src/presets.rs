// use reqwest
use std::error::Error;
use std::fs::File;
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

/// Downloads a given file
fn get_file(path: &Path, url: &str) -> Result<(), Box<dyn Error>> {
    println!("GET file");
    let mut res = reqwest::blocking::get(url)?;
    println!("Status: {}", res.status());
    let mut out = File::create(path).expect("failed to create file");
    std::io::copy(&mut res, &mut out).expect("failed to copy content");
    println!("Tar ball downloaded");
    Ok(())
}

/// Unpacks a tar ball to a new directory
fn unpack_tar(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    println!("Unpacking tar ball {:?}", &src);
    let tar_gz = File::open(src)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    println!("Done");
    Ok(())
}

pub fn match_preset(preset: Option<&str>) -> Result<(), Box<dyn Error>> {
    match preset {
        None => (),
        Some(_) => {
            let config_path = flowy::get_config_dir()?;

            let mut archive_path = config_path.clone();
            archive_path.push("lake.tar.gz");
            let mut dir_path = config_path.clone();
            dir_path.push("lake");

            // Download and unzip the folder
            get_file(
                &archive_path,
                "https://bucket-more.s3.ap-south-1.amazonaws.com/uploads/lake.tar.gz",
            )?;
            unpack_tar(&archive_path, &config_path).unwrap();

            // Deleting the tar ball
            std::fs::remove_file(&archive_path)?;

            // A config file, config.toml must be generated now
            flowy::generate_config(&dir_path)?;

            println!("Preset set successfully")
        }
    }

    Ok(())
}
