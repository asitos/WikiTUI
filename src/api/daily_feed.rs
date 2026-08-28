use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSummary {
    pub title: String,
    pub normalizedtitle: Option<String>,
    pub description: Option<String>,
    pub extract: Option<String>,
}

impl PageSummary {
    pub fn display_title(&self) -> &str {
        self.normalizedtitle
            .as_deref()
            .unwrap_or(self.title.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub story: Option<String>,
    #[serde(default)]
    pub links: Vec<PageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnThisDayEvent {
    pub text: String,
    pub year: Option<i32>,
    #[serde(default)]
    pub pages: Vec<PageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostReadArticle {
    pub title: String,
    pub normalizedtitle: Option<String>,
    pub description: Option<String>,
    pub extract: Option<String>,
    pub views: Option<u64>,
    pub rank: Option<u32>,
}

impl MostReadArticle {
    pub fn display_title(&self) -> &str {
        self.normalizedtitle
            .as_deref()
            .unwrap_or(self.title.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostReadPayload {
    pub date: Option<String>,
    #[serde(default)]
    pub articles: Vec<MostReadArticle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyFeed {
    pub tfa: Option<PageSummary>,
    #[serde(default)]
    pub news: Vec<NewsItem>,
    #[serde(default)]
    pub onthisday: Vec<OnThisDayEvent>,
    pub mostread: Option<MostReadPayload>,
}

pub fn utc_today() -> (u32, u32, u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = (now / 86400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1020 + doe / 1460 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as u32, m, d)
}

pub fn feed_cache_path(year: u32, month: u32, day: u32) -> PathBuf {
    crate::paths::cache_dir().join(format!("feed_{}_{:02}_{:02}.json", year, month, day))
}

pub fn get_cached_daily_feed(year: u32, month: u32, day: u32) -> Option<DailyFeed> {
    let path = feed_cache_path(year, month, day);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_cached_daily_feed(year: u32, month: u32, day: u32, feed: &DailyFeed) {
    let path = feed_cache_path(year, month, day);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(feed) {
        let _ = fs::write(path, json);
    }
}

pub fn fetch_daily_feed(
    agent: &ureq::Agent,
    timeout: u64,
    offline_cache: bool,
) -> Result<DailyFeed, String> {
    let (year, month, day) = utc_today();

    if offline_cache {
        if let Some(cached) = get_cached_daily_feed(year, month, day) {
            return Ok(cached);
        }
    }

    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/feed/featured/{:04}/{:02}/{:02}",
        year, month, day
    );

    let resp = agent
        .get(&url)
        .timeout(std::time::Duration::from_secs(timeout))
        .call()
        .map_err(|e| format!("Failed to fetch daily feed: {}", e))?;

    let feed: DailyFeed = resp
        .into_json()
        .map_err(|e| format!("Failed to parse daily feed: {}", e))?;

    if offline_cache {
        save_cached_daily_feed(year, month, day, &feed);
    }

    Ok(feed)
}