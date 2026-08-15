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

pub async fn fetch_random_article(client: &reqwest::Client) -> Result<(String, String), String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("list", "random"),
            ("rnnamespace", "0"),
            ("rnlimit", "1"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let rand_resp: WikiRandomResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

    let title = rand_resp
        .query
        .and_then(|q| q.random.into_iter().next())
        .map(|r| r.title)
        .ok_or_else(|| "no random article returned".to_string())?;

    let content = fetch_article_wikipedia(client, &title).await?;
    Ok((title, content))
}
