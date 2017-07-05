use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub fn load_file(path: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
