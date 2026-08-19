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
    pub show_footnotes: bool,
    pub show_external_links: bool,
    pub toc_section_numbers: bool,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            scroll_lines: 1,
            underline_links: false,
            show_footnotes: true,
            show_external_links: true,
            toc_section_numbers: true,
        }
    }
}

use std::time::SystemTime;

impl Config {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            dir.push("config.toml");
            dir
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn get_modified_time() -> Option<SystemTime> {
        let path = Self::config_path();
        fs::metadata(&path).ok().and_then(|m| m.modified().ok())
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

    pub fn reload_if_changed(&mut self, last_mtime: &mut Option<SystemTime>) -> bool {
        let current_mtime = Self::get_modified_time();
        if current_mtime.is_some() && current_mtime != *last_mtime {
            *last_mtime = current_mtime;
            let path = Self::config_path();
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(new_config) = toml::from_str::<Config>(&content) {
                    *self = new_config;
                    return true;
                }
            }
        }
        false
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}
