use std::path::Path;

pub fn find_filename(file: &Path) -> String {
    file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .split(" [")
        .next()
        .unwrap_or("Unknown")
        .to_string()
}