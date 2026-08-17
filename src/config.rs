use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub reader: ReaderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub liked_readonly: bool,
    pub auto_restore_session: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            liked_readonly: true,
            auto_restore_session: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReaderConfig {
    pub scroll_lines: usize,
    pub underline_links: bool,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            scroll_lines: 1,
            underline_links: false,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            let _ = fs::create_dir_all(&dir);
            dir.push("config.toml");
            dir
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
        let config = Self::default();
        config.save();
        config
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}
