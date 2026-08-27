use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("wikid");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".config").join("wikid");
        }
    }
    PathBuf::from(".")
}

pub fn cache_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("WIKID_CACHE_DIR") {
        if !override_dir.trim().is_empty() {
            return PathBuf::from(override_dir);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("wikid");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".cache").join("wikid");
        }
    }
    PathBuf::from("cache")
}