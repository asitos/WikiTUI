use crate::feed::algorithm::FeedItem;
use rand::Rng;
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

pub async fn fetch_feed_batch(client: &reqwest::Client) -> Result<Vec<FeedItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let profile = crate::feed::profile::FeedProfile::load();

    let active_subcats = profile.get_active_subcategories();
    let mut selected_cat = None;
    if !active_subcats.is_empty() {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0..100) < 80 {
            let idx = rng.gen_range(0..active_subcats.len());
            selected_cat = Some(active_subcats[idx].clone());
        }
    }

    let res = if let Some(cat_name) = selected_cat {
        let category_title = format!("Category:{}", cat_name);
        client
            .get(url)
            .query(&[
                ("action", "query"),
                ("generator", "categorymembers"),
                ("gcmtitle", &category_title),
                ("gcmtype", "page"),
                ("gcmlimit", "10"),
                ("prop", "description|extracts|categories"),
                ("exintro", "1"),
                ("explaintext", "1"),
                ("clshow", "!hidden"),
                ("cllimit", "15"),
                ("format", "json"),
            ])
            .send()
            .await
    } else {
        client
            .get(url)
            .query(&[
                ("action", "query"),
                ("generator", "random"),
                ("grnnamespace", "0"),
                ("grnlimit", "5"),
                ("prop", "description|extracts|categories"),
                ("exintro", "1"),
                ("explaintext", "1"),
                ("clshow", "!hidden"),
                ("cllimit", "15"),
                ("format", "json"),
            ])
            .send()
            .await
    };

    let res = res.map_err(|e| format!("network error: {}", e))?;

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
