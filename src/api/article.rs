use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Deserialize)]
struct WikiParseText {
    #[serde(rename = "*")]
    html: Option<String>,
}

#[derive(Deserialize)]
struct WikiParseObject {
    text: Option<WikiParseText>,
}

#[derive(Deserialize)]
struct WikiParseResponse {
    parse: Option<WikiParseObject>,
}

pub fn cache_dir() -> PathBuf {
    if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
        let mut p = PathBuf::from(cache_home);
        p.push("wikid");
        p.push("articles");
        p
    } else if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".cache");
        p.push("wikid");
        p.push("articles");
        p
    } else {
        PathBuf::from(".cache/wikid/articles")
    }
}

pub fn cache_file_path(title: &str) -> PathBuf {
    let safe_name: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    title.hash(&mut hasher);
    let hash = hasher.finish();
    cache_dir().join(format!("{}_{:016x}.html", safe_name, hash))
}

pub fn get_cached_article(title: &str) -> Option<String> {
    let path = cache_file_path(title);
    std::fs::read_to_string(&path).ok()
}

pub fn save_cached_article(title: &str, html: &str) {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = cache_file_path(title);
    let _ = std::fs::write(&path, html);
}

pub fn fetch_article_wikipedia(agent: &ureq::Agent, title: &str) -> Result<String, String> {
    if let Some(cached_html) = get_cached_article(title) {
        return Ok(cached_html);
    }

    let decoded_title = crate::parser::url_decode(title).replace('_', " ");
    let url = "https://en.wikipedia.org/w/api.php";
    let res = agent
        .get(url)
        .query("action", "parse")
        .query("page", &decoded_title)
        .query("prop", "text")
        .query("format", "json")
        .query("disableeditsection", "1")
        .query("disabletoc", "1")
        .query("redirects", "1")
        .call();

    match res {
        Ok(response) => {
            let parse_resp: WikiParseResponse =
                response.into_json().map_err(|e| format!("parse error: {}", e))?;

            let html = parse_resp
                .parse
                .and_then(|p| p.text)
                .and_then(|t| t.html)
                .filter(|h| !h.trim().is_empty());

            if let Some(h) = html {
                save_cached_article(title, &h);
                Ok(h)
            } else {
                Err("article HTML content not found".to_string())
            }
        }
        Err(err) => {
            if let Some(cached) = get_cached_article(title) {
                return Ok(cached);
            }
            Err(format!("network error: {}", err))
        }
    }
}
