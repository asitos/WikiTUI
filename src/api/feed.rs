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

async fn fetch_category_items(
    client: &reqwest::Client,
    category: &str,
) -> Result<Vec<FeedItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let category_title = format!("Category:{}", category);
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("generator", "categorymembers"),
            ("gcmtitle", &category_title),
            ("gcmtype", "page"),
            ("gcmlimit", "2"),
            ("prop", "description|extracts|categories"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("clshow", "!hidden"),
            ("cllimit", "15"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let feed_resp: WikiFeedResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

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

async fn fetch_random_items(client: &reqwest::Client) -> Result<Vec<FeedItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("generator", "random"),
            ("grnnamespace", "0"),
            ("grnlimit", "3"),
            ("prop", "description|extracts|categories"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("clshow", "!hidden"),
            ("cllimit", "15"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let feed_resp: WikiFeedResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

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

pub async fn fetch_feed_batch(client: &reqwest::Client) -> Result<Vec<FeedItem>, String> {
    let profile = crate::feed::profile::FeedProfile::load();
    let active_subcats = profile.get_active_subcategories();

    let mut chosen_cats = Vec::new();
    if !active_subcats.is_empty() {
        let mut available = active_subcats.clone();
        fastrand::shuffle(&mut available);
        chosen_cats = available.into_iter().take(3).collect();
    }

    let mut items = Vec::new();
    let mut tasks = Vec::new();

    for cat in chosen_cats {
        let client_ref = client.clone();
        tasks.push(tokio::spawn(async move {
            fetch_category_items(&client_ref, &cat).await
        }));
    }

    let client_ref = client.clone();
    tasks.push(tokio::spawn(async move {
        fetch_random_items(&client_ref).await
    }));

    for task in tasks {
        if let Ok(Ok(batch)) = task.await {
            items.extend(batch);
        }
    }

    fastrand::shuffle(&mut items);
    Ok(items)
}
