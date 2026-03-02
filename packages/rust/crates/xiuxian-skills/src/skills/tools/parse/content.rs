use std::fs;
use std::path::Path;

pub(super) fn read_script_content(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) => match fs::read(path) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to read file as UTF-8: {error}"),
            ))),
        },
    }
}
