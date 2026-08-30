use std::fs;
use std::path::PathBuf;

pub fn get_cached_image_path(url: &str) -> Option<PathBuf> {
    let path = image_cache_path(url);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn save_cached_image(url: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    let path = image_cache_path(url);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    Ok(path)
}

pub fn image_cache_path(url: &str) -> PathBuf {
    let cache_dir = crate::paths::image_cache_dir();
    let hash = fnv1a_hash_hex(url.as_bytes());
    let ext = url
        .split('?')
        .next()
        .and_then(|u| u.rsplit('.').next())
        .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("img");
    cache_dir.join(format!("{}.{}", hash, ext))
}

fn fnv1a_hash_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}
