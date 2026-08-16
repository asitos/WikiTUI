use super::article::fetch_article_wikipedia;
use serde::Deserialize;

#[derive(Deserialize)]
struct WikiRandomItem {
    title: String,
}

#[derive(Deserialize)]
struct WikiRandomQuery {
    random: Vec<WikiRandomItem>,
}

#[derive(Deserialize)]
struct WikiRandomResponse {
    query: Option<WikiRandomQuery>,
}

pub fn fetch_random_article(agent: &ureq::Agent) -> Result<(String, String), String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = agent
        .get(url)
        .query("action", "query")
        .query("list", "random")
        .query("rnnamespace", "0")
        .query("rnlimit", "1")
        .query("format", "json")
        .call()
        .map_err(|e| format!("network error: {}", e))?;

    let rand_resp: WikiRandomResponse = res
        .into_json()
        .map_err(|e| format!("parse error: {}", e))?;

    let title = rand_resp
        .query
        .and_then(|q| q.random.into_iter().next())
        .map(|r| r.title)
        .ok_or_else(|| "no random article returned".to_string())?;

    let content = fetch_article_wikipedia(agent, &title)?;
    Ok((title, content))
}
