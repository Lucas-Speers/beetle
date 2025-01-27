use std::{env, path::Path};

use crate::MainError;

pub fn read_full_file(path: &Path) -> Result<String, MainError> {

    if !path.exists() {
        return Err(MainError::FileNotFound);
    }

    let content = std::fs::read(path).map_err(|_| MainError::FileNotFound)?;

    let string = String::from_utf8(content).map_err(|_| MainError::UnicodeError)?;

    Ok(string)
}