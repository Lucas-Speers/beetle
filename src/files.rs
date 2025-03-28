use std::path::Path;

pub fn read_full_file(path: &Path) -> Result<String, ()> {

    if !path.exists() {
        println!("File: `{:?}` not found", path);
        return Err(());
    }

    if let Ok(content) = std::fs::read(path) {
        if let Ok(string) = String::from_utf8(content) {
            return Ok(string);
        }
    }

    Err(())
}