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

pub fn search_wikipedia(
    agent: &ureq::Agent,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResultItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let limit_str = limit.clamp(1, 50).to_string();
    let res = agent
        .get(url)
        .query("action", "query")
        .query("generator", "search")
        .query("gsrsearch", query)
        .query("gsrlimit", &limit_str)
        .query("prop", "description")
        .query("format", "json")
        .call()
        .map_err(|e| format!("network error: {}", e))?;

    let search_resp: WikiGenSearchResponse =
        res.into_json().map_err(|e| format!("parse error: {}", e))?;

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
