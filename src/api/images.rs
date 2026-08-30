use crate::graphics::cache::{get_cached_image_path, save_cached_image};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

pub fn fetch_and_cache_image(
    agent: &ureq::Agent,
    url: &str,
    timeout_secs: u64,
) -> Result<PathBuf, String> {
    if let Some(path) = get_cached_image_path(url) {
        return Ok(path);
    }

    let response = agent
        .get(url)
        .timeout(Duration::from_secs(timeout_secs.max(5)))
        .call()
        .map_err(|e| e.to_string())?;

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;

    save_cached_image(url, &bytes).map_err(|e| e.to_string())
}
