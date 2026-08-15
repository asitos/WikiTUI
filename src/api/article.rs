use serde::Deserialize;

#[derive(Deserialize)]
struct WikiParseText {
    #[serde(rename = "*")]
    html: Option<String>,
}

#[derive(Deserialize)]
struct WikiParseObject {
    text: Option<WikiParseText>,
}

#[derive(Deserialize)]
struct WikiParseResponse {
    parse: Option<WikiParseObject>,
}

pub async fn fetch_article_wikipedia(
    client: &reqwest::Client,
    title: &str,
) -> Result<String, String> {
    let decoded_title = crate::parser::url_decode(title).replace('_', " ");
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "parse"),
            ("page", &decoded_title),
            ("prop", "text"),
            ("format", "json"),
            ("disableeditsection", "1"),
            ("disabletoc", "1"),
            ("redirects", "1"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let parse_resp: WikiParseResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

    let html = parse_resp
        .parse
        .and_then(|p| p.text)
        .and_then(|t| t.html)
        .filter(|h| !h.trim().is_empty());

    if let Some(h) = html {
        Ok(h)
    } else {
        Err("article HTML content not found".to_string())
    }
}
