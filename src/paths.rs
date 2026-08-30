use std::path::PathBuf;

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var(var)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
}

pub fn config_dir() -> PathBuf {
    env_path("XDG_CONFIG_HOME")
        .map(|p| p.join("wikid"))
        .or_else(|| env_path("HOME").map(|p| p.join(".config").join("wikid")))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn cache_dir() -> PathBuf {
    env_path("WIKID_CACHE_DIR")
        .or_else(|| env_path("XDG_CACHE_HOME").map(|p| p.join("wikid")))
        .or_else(|| env_path("HOME").map(|p| p.join(".cache").join("wikid")))
        .unwrap_or_else(|| PathBuf::from("cache"))
}

pub fn audio_cache_dir() -> PathBuf {
    cache_dir().join("audio")
}
