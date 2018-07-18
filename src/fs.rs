use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn read_file<T: AsRef<Path>>(path: T) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn write_file<T: AsRef<Path>>(path: T, text: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    file.write_all(text.as_ref())?;
    Ok(())
}
