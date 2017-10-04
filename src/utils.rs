use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn load_file<T: AsRef<Path>>(path: T) -> Result<String, Box<Error>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
