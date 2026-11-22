use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// File or directory containing only real/source files
    Real,
    /// File or directory containing only temporary files
    Temporary,
    /// Directory containing both real and temporary files
    Mixed,
}

impl FileType {
    /// Calculate the file type based on real and temp sizes
    pub fn from_sizes(real_size: u64, temp_size: u64) -> Self {
        match (real_size > 0, temp_size > 0) {
            (true, true) => FileType::Mixed,
            (true, false) => FileType::Real,
            (false, true) => FileType::Temporary,
            (false, false) => FileType::Real, // Empty defaults to Real
        }
    }

    /// Get the ratio of temporary files (0.0 to 1.0)
    pub fn temp_ratio(real_size: u64, temp_size: u64) -> f64 {
        let total = real_size + temp_size;
        if total == 0 {
            0.0
        } else {
            temp_size as f64 / total as f64
        }
    }
}
