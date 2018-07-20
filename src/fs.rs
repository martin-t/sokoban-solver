use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

crate fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// will be useful when writing out results
// also used in tests for updating results but usually commented out
#[allow(unused)]
crate fn write_file<P: AsRef<Path>>(path: P, text: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    file.write_all(text.as_ref())?;
    Ok(())
}
