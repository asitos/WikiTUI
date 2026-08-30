use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub fn check_latest_release(
    agent: &ureq::Agent,
    timeout_secs: u64,
) -> Result<String, super::ApiError> {
    let url = "https://api.github.com/repos/sharkthakftw/wikid/releases/latest";
    let res = agent
        .get(url)
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .call()
        .map_err(|e| super::ApiError::Network(e.to_string()))?;

    let release: GitHubRelease = res
        .into_json()
        .map_err(|e| super::ApiError::Parse(e.to_string()))?;

    Ok(release.tag_name)
}
