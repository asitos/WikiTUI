use super::SearchResultItem;
use serde::Deserialize;

#[derive(Deserialize)]
struct WikiPageDescription {
    title: String,
    description: Option<String>,
    index: Option<i32>,
}

#[derive(Deserialize)]
struct WikiGenSearchQuery {
    pages: Option<std::collections::HashMap<String, WikiPageDescription>>,
}

#[derive(Deserialize)]
struct WikiGenSearchResponse {
    query: Option<WikiGenSearchQuery>,
}

pub async fn search_wikipedia(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<SearchResultItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("generator", "search"),
            ("gsrsearch", query),
            ("gsrlimit", "30"),
            ("prop", "description"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let search_resp: WikiGenSearchResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

    let mut items = Vec::new();
    if let Some(q) = search_resp.query {
        if let Some(pages) = q.pages {
            let mut page_list: Vec<_> = pages.into_values().collect();
            page_list.sort_by_key(|p| p.index.unwrap_or(9999));
            for item in page_list {
                let desc = item
                    .description
                    .filter(|d| !d.trim().is_empty())
                    .unwrap_or_default();

                items.push(SearchResultItem {
                    title: item.title,
                    snippet: desc,
                });
            }
        }
    }

    Ok(items)
}
