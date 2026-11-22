use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_temp_patterns")]
    pub temp_patterns: Vec<String>,

    #[serde(default = "default_min_display_size")]
    pub min_display_size: u64,

    #[serde(default)]
    pub colors: ColorScheme,

    #[serde(default)]
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    #[serde(default = "default_real_file_fg")]
    pub real_file_fg: String,

    #[serde(default = "default_temp_file_fg")]
    pub temp_file_fg: String,

    #[serde(default = "default_selected_border")]
    pub selected_border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    #[serde(default = "default_parallelism")]
    pub parallelism: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            temp_patterns: default_temp_patterns(),
            min_display_size: default_min_display_size(),
            colors: ColorScheme::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            real_file_fg: default_real_file_fg(),
            temp_file_fg: default_temp_file_fg(),
            selected_border: default_selected_border(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
            parallelism: default_parallelism(),
        }
    }
}

fn default_temp_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".pycache".to_string(),
        "__pycache__".to_string(),
        "target".to_string(),
        "build".to_string(),
        "dist".to_string(),
        ".git".to_string(),
        ".cache".to_string(),
        "*.tmp".to_string(),
        "*.log".to_string(),
        ".pytest_cache".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        ".DS_Store".to_string(),
        "Thumbs.db".to_string(),
    ]
}

fn default_min_display_size() -> u64 {
    1_048_576 // 1 MB
}

fn default_real_file_fg() -> String {
    "#0066CC".to_string()
}

fn default_temp_file_fg() -> String {
    "#99CCFF".to_string()
}

fn default_selected_border() -> String {
    "#FFFF00".to_string()
}

fn default_max_depth() -> usize {
    10
}

fn default_parallelism() -> usize {
    4
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try current directory first
        if let Ok(config) = Self::load_from_path("./.declutter.toml") {
            return Ok(config);
        }

        // Try home directory
        if let Some(home) = dirs::home_dir() {
            let home_config = home.join(".declutter.toml");
            if let Ok(config) = Self::load_from_path(&home_config) {
                return Ok(config);
            }
        }

        // Fallback to defaults
        Ok(Config::default())
    }

    fn load_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

// Add dirs dependency for home_dir
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
