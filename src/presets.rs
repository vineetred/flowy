// use reqwest
use flate2::read::GzDecoder;
use std::error::Error;
use tar::Archive;

fn get_file(url: &str) -> Result<(), Box<dyn Error>> {
    println!("GET file");
    let mut res = reqwest::blocking::get(url)?;
    println!("Status: {}", res.status());
    let mut out = std::fs::File::create("lake.tar.gz").expect("failed to create file");
    std::io::copy(&mut res, &mut out).expect("failed to copy content");
    println!("lake.tar.gz downloaded");
    Ok(())
}

fn unzip_tar(path: &str) -> Result<(), Box<dyn Error>> {
    println!("Unzipping tar ball {}", &path);
    let tar_gz = std::fs::File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;
    println!("Done");
    Ok(())
}

fn delete_tar(path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::remove_file(path)?;
    Ok(())
}
pub fn match_preset(preset: Option<&str>) -> Result<(), Box<dyn Error>> {
    match preset {
        None => (),
        Some(_) => {
            // Download and unzip the folder
            get_file("https://bucket-more.s3.ap-south-1.amazonaws.com/uploads/lake.tar.gz")?;
            unzip_tar("lake.tar.gz").unwrap();
            // Deleting the tar ball
            delete_tar("lake.tar.gz")?;
            // Find the current dir and pass it to the generate config
            let mut current_dir = std::env::current_dir()?.display().to_string();
            current_dir.push_str("/lake");
            // A config file, times.toml must be generated now
            flowy::generate_config(&current_dir)?;
            println!("Preset set successfully")
        }
    }

    Ok(())
}
