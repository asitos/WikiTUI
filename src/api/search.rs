use super::SearchResultItem;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct WikiPageCategory {
    title: String,
}

#[derive(Deserialize)]
struct WikiPageDescription {
    title: String,
    description: Option<String>,
    index: Option<i32>,
    categories: Option<Vec<WikiPageCategory>>,
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
    timeout_secs: u64,
) -> Result<Vec<SearchResultItem>, super::ApiError> {
    let url = "https://en.wikipedia.org/w/api.php";
    let limit_str = limit.clamp(1, 50).to_string();
    let res = agent
        .get(url)
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .query("action", "query")
        .query("generator", "search")
        .query("gsrsearch", query)
        .query("gsrlimit", &limit_str)
        .query("prop", "description|categories")
        .query(
            "clcategories",
            "Category:Spoken_Wikipedia_articles|Category:Spoken_articles",
        )
        .query("format", "json")
        .call()
        .map_err(|e| super::ApiError::Network(e.to_string()))?;

    let search_resp: WikiGenSearchResponse = res
        .into_json()
        .map_err(|e| super::ApiError::Parse(e.to_string()))?;

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
                let has_audio = item.categories.as_ref().is_some_and(|c| !c.is_empty())
                    || item.title.starts_with("Spoken:")
                    || desc.to_lowercase().contains("spoken wikipedia");

                items.push(SearchResultItem {
                    title: item.title,
                    snippet: desc,
                    has_audio,
                });
            }
        }
    }

    Ok(items)
}
