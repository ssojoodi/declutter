use std::fs;
use std::path::Path;

pub fn calculate_file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0)
}
