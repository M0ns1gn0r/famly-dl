use std::fs;
use std::path::Path;

pub fn create_dir(name: &str) -> std::io::Result<()> {
    let path = Path::new(name);
    if !std::path::Path::exists(&path) {
        fs::create_dir(path)
    } else {
        Ok(())
    }
}