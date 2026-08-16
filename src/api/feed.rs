use crate::feed::algorithm::FeedItem;
use serde::Deserialize;

#[derive(Deserialize)]
struct WikiCategoryItem {
    title: String,
}

#[derive(Deserialize)]
struct WikiPageProp {
    title: Option<String>,
    description: Option<String>,
    extract: Option<String>,
    categories: Option<Vec<WikiCategoryItem>>,
}

#[derive(Deserialize)]
struct WikiFeedQuery {
    pages: Option<std::collections::HashMap<String, WikiPageProp>>,
}

#[derive(Deserialize)]
struct WikiFeedResponse {
    query: Option<WikiFeedQuery>,
}

fn fetch_category_items(agent: &ureq::Agent, category: &str) -> Result<Vec<FeedItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let category_title = format!("Category:{}", category);
    let res = agent
        .get(url)
        .query("action", "query")
        .query("generator", "categorymembers")
        .query("gcmtitle", &category_title)
        .query("gcmtype", "page")
        .query("gcmlimit", "2")
        .query("prop", "description|extracts|categories")
        .query("exintro", "1")
        .query("explaintext", "1")
        .query("clshow", "!hidden")
        .query("cllimit", "15")
        .query("format", "json")
        .call()
        .map_err(|e| format!("network error: {}", e))?;

    let feed_resp: WikiFeedResponse = res.into_json().map_err(|e| format!("parse error: {}", e))?;

    let mut items = Vec::new();
    if let Some(query) = feed_resp.query {
        if let Some(pages) = query.pages {
            for (_, page) in pages {
                if let Some(title) = page.title {
                    let short_description = page.description.filter(|d| !d.trim().is_empty());
                    let snippet = page.extract.unwrap_or_default().trim().to_string();
                    let mut categories: Vec<String> = page
                        .categories
                        .unwrap_or_default()
                        .into_iter()
                        .map(|c| {
                            if let Some(stripped) = c.title.strip_prefix("Category:") {
                                stripped.to_string()
                            } else {
                                c.title
                            }
                        })
                        .filter(|cat| {
                            let lower = cat.to_lowercase();
                            !lower.starts_with("all ")
                                && !lower.starts_with("articles ")
                                && !lower.starts_with("cs1 ")
                                && !lower.contains("stubs")
                                && !lower.contains("tracking")
                        })
                        .collect();

                    if categories.is_empty() {
                        categories = title
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| w.len() > 3)
                            .map(|w| w.to_lowercase())
                            .take(3)
                            .collect();
                    }

                    items.push(FeedItem {
                        title,
                        short_description,
                        snippet,
                        categories,
                        is_liked: false,
                    });
                }
            }
        }
    }
    Ok(items)
}

fn fetch_random_items(agent: &ureq::Agent) -> Result<Vec<FeedItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = agent
        .get(url)
        .query("action", "query")
        .query("generator", "random")
        .query("grnnamespace", "0")
        .query("grnlimit", "3")
        .query("prop", "description|extracts|categories")
        .query("exintro", "1")
        .query("explaintext", "1")
        .query("clshow", "!hidden")
        .query("cllimit", "15")
        .query("format", "json")
        .call()
        .map_err(|e| format!("network error: {}", e))?;

    let feed_resp: WikiFeedResponse = res.into_json().map_err(|e| format!("parse error: {}", e))?;

    let mut items = Vec::new();
    if let Some(query) = feed_resp.query {
        if let Some(pages) = query.pages {
            for (_, page) in pages {
                if let Some(title) = page.title {
                    let short_description = page.description.filter(|d| !d.trim().is_empty());
                    let snippet = page.extract.unwrap_or_default().trim().to_string();
                    let mut categories: Vec<String> = page
                        .categories
                        .unwrap_or_default()
                        .into_iter()
                        .map(|c| {
                            if let Some(stripped) = c.title.strip_prefix("Category:") {
                                stripped.to_string()
                            } else {
                                c.title
                            }
                        })
                        .filter(|cat| {
                            let lower = cat.to_lowercase();
                            !lower.starts_with("all ")
                                && !lower.starts_with("articles ")
                                && !lower.starts_with("cs1 ")
                                && !lower.contains("stubs")
                                && !lower.contains("tracking")
                        })
                        .collect();

                    if categories.is_empty() {
                        categories = title
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| w.len() > 3)
                            .map(|w| w.to_lowercase())
                            .take(3)
                            .collect();
                    }

                    items.push(FeedItem {
                        title,
                        short_description,
                        snippet,
                        categories,
                        is_liked: false,
                    });
                }
            }
        }
    }
    Ok(items)
}

pub fn fetch_feed_batch(agent: &ureq::Agent) -> Result<Vec<FeedItem>, String> {
    let profile = crate::feed::profile::FeedProfile::load();
    let active_subcats = profile.get_active_subcategories();

    let mut chosen_cats = Vec::new();
    if !active_subcats.is_empty() {
        let mut available = active_subcats.clone();
        fastrand::shuffle(&mut available);
        chosen_cats = available.into_iter().take(3).collect();
    }

    let mut items = Vec::new();
    let mut handles = Vec::new();

    for cat in chosen_cats {
        let agent = agent.clone();
        handles.push(std::thread::spawn(move || {
            fetch_category_items(&agent, &cat)
        }));
    }

    let agent_rand = agent.clone();
    handles.push(std::thread::spawn(move || fetch_random_items(&agent_rand)));

    for handle in handles {
        if let Ok(Ok(batch)) = handle.join() {
            items.extend(batch);
        }
    }

    fastrand::shuffle(&mut items);
    Ok(items)
}
