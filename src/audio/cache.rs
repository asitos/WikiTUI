use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_cached_duration(url: &str) -> Option<u64> {
    let path = crate::paths::audio_cache_dir().join("durations.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, u64>>(&content) {
            return map.get(url).copied();
        }
    }
    None
}

pub fn save_cached_duration(url: &str, duration_secs: u64) {
    let dir = crate::paths::audio_cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("durations.json");
    let mut map = std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<HashMap<String, u64>>(&c).ok())
        .unwrap_or_default();
    map.insert(url.to_string(), duration_secs);
    if let Ok(json) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn safe_audio_filename(url: &str) -> String {
    let base = if let Some(idx) = url.rfind('/') {
        let mut name = &url[idx + 1..];
        if let Some(q) = name.find('?') {
            name = &name[..q];
        }
        name
    } else {
        "track.ogg"
    };
    let sanitized: String = base
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "track.ogg".to_string()
    } else {
        sanitized
    }
}

pub fn get_cached_audio_path(url: &str) -> Option<PathBuf> {
    let filename = safe_audio_filename(url);
    let path = crate::paths::audio_cache_dir().join(&filename);
    if path.exists() && std::fs::metadata(&path).map(|m| m.len() > 1024).unwrap_or(false) {
        Some(path)
    } else {
        None
    }
}

pub fn spawn_background_audio_download(url: &str) {
    let url_str = url.to_string();
    let filename = safe_audio_filename(url);
    let dir = crate::paths::audio_cache_dir();
    let final_path = dir.join(&filename);
    if final_path.exists() {
        return;
    }

    std::thread::spawn(move || {
        let _ = std::fs::create_dir_all(&dir);
        let part_path = dir.join(format!("{}.part", filename));
        const USER_AGENT: &str = concat!(
            "wikid/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/sharkthakftw/wikid)"
        );

        if let Ok(resp) = ureq::get(&url_str).set("User-Agent", USER_AGENT).call() {
            if let Ok(mut file) = std::fs::File::create(&part_path) {
                let mut reader = resp.into_reader();
                if std::io::copy(&mut reader, &mut file).is_ok() {
                    let _ = std::fs::rename(&part_path, &final_path);
                } else {
                    let _ = std::fs::remove_file(&part_path);
                }
            }
        }
    });
}
