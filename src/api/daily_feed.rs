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
pub struct OngoingItem {
    pub display: String,
    pub target: String,
    #[serde(default)]
    pub sub_events: Vec<(String, String)>, // (target, display)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDeathItem {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyFeed {
    pub tfa: Option<PageSummary>,
    #[serde(default)]
    pub news: Vec<NewsItem>,
    #[serde(default)]
    pub onthisday: Vec<OnThisDayEvent>,
    pub mostread: Option<MostReadPayload>,
    #[serde(default)]
    pub ongoing: Vec<OngoingItem>,
    #[serde(default)]
    pub recent_deaths: Vec<RecentDeathItem>,
}

pub fn strip_wikitext_comments(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' && chars.clone().take(3).collect::<String>() == "!--" {
            chars.next();
            chars.next();
            chars.next();
            let mut dashes = 0;
            for nc in chars.by_ref() {
                if nc == '-' {
                    dashes += 1;
                } else if nc == '>' && dashes >= 2 {
                    break;
                } else {
                    dashes = 0;
                }
            }
            continue;
        }
        res.push(c);
    }
    res
}

pub fn extract_all_links_from_wikitext(s: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("[[") {
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("]]") {
            let link_content = &after_start[..end];
            let parts: Vec<&str> = link_content.split('|').collect();
            if parts.len() >= 2 {
                links.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
            } else {
                let p = parts[0].trim().to_string();
                links.push((p.clone(), p));
            }
            rest = &after_start[end + 2..];
        } else {
            break;
        }
    }
    links
}

pub fn parse_itn_footer(wikitext: &str) -> (Vec<OngoingItem>, Vec<RecentDeathItem>) {
    let mut ongoing = Vec::new();
    let mut recent_deaths = Vec::new();
    let clean = strip_wikitext_comments(wikitext);

    let currentevents_block = if let Some(pos) = clean.find("currentevents") {
        let rest = &clean[pos + 13..];
        if let Some(d_pos) = rest.find("recentdeaths") {
            &rest[..d_pos]
        } else {
            rest
        }
    } else {
        ""
    };

    let recentdeaths_block = if let Some(pos) = clean.find("recentdeaths") {
        &clean[pos + 12..]
    } else {
        ""
    };

    let mut current_ongoing: Option<OngoingItem> = None;
    for line in currentevents_block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") {
            for (target, display) in extract_all_links_from_wikitext(trimmed) {
                if let Some(og) = &mut current_ongoing {
                    og.sub_events.push((target, display));
                }
            }
        } else if trimmed.starts_with('*') {
            if let Some(og) = current_ongoing.take() {
                ongoing.push(og);
            }
            let clean_line = trimmed.trim_start_matches('*');
            let mut sub_events = Vec::new();
            let mut main_link = None;

            for link in extract_all_links_from_wikitext(clean_line) {
                if main_link.is_none() {
                    main_link = Some(link);
                } else {
                    sub_events.push(link);
                }
            }

            if let Some((target, display)) = main_link {
                current_ongoing = Some(OngoingItem {
                    display,
                    target,
                    sub_events,
                });
            }
        }
    }
    if let Some(og) = current_ongoing {
        ongoing.push(og);
    }

    for line in recentdeaths_block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('*') {
            let clean_line = trimmed.trim_start_matches('*');
            for (target, display) in extract_all_links_from_wikitext(clean_line) {
                recent_deaths.push(RecentDeathItem {
                    name: display,
                    target,
                });
            }
        }
    }

    (ongoing, recent_deaths)
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

    let mut feed: DailyFeed = resp
        .into_json()
        .map_err(|e| format!("Failed to parse daily feed: {}", e))?;

    // Also fetch ITN footer for Ongoing and Recent Deaths
    let itn_url = "https://en.wikipedia.org/w/api.php?action=parse&page=Template:In_the_news&prop=wikitext&format=json";
    if let Ok(itn_resp) = agent
        .get(itn_url)
        .timeout(std::time::Duration::from_secs(timeout))
        .call()
    {
        if let Ok(itn_json) = itn_resp.into_json::<serde_json::Value>() {
            if let Some(wikitext) = itn_json
                .get("parse")
                .and_then(|p| p.get("wikitext"))
                .and_then(|t| t.get("*"))
                .and_then(|h| h.as_str())
            {
                let (ongoing, recent_deaths) = parse_itn_footer(wikitext);
                feed.ongoing = ongoing;
                feed.recent_deaths = recent_deaths;
            }
        }
    }

    if offline_cache {
        save_cached_daily_feed(year, month, day, &feed);
    }

    Ok(feed)
}