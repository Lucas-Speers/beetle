use std::path::Path;

use anyhow::{Context, Ok, Result};

pub fn read_full_file(path: &Path) -> Result<String> {

    let content = std::fs::read(path)
        .with_context(|| format!("File could not be read: \'{}\'", path.display()))?;

    let string = String::from_utf8(content)
        .with_context(|| format!("File was not UTF-8: \'{}\'", path.display()))?;

    Ok(string)
}