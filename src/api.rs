use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub title: String,
    pub snippet: String,
}

pub enum NetworkCommand {
    Search { pane_id: usize, query: String },
    FetchArticle { pane_id: usize, title: String },
}

pub enum NetworkEvent {
    SearchResult {
        pane_id: usize,
        query: String,
        results: Vec<SearchResultItem>,
    },
    ArticleResult {
        pane_id: usize,
        title: String,
        content: String,
    },
    Error {
        pane_id: usize,
        message: String,
    },
}

#[derive(Deserialize)]
struct WikiSearchQuery {
    search: Vec<WikiSearchResult>,
}

#[derive(Deserialize)]
struct WikiSearchResult {
    title: String,
    snippet: String,
}

#[derive(Deserialize)]
struct WikiSearchResponse {
    query: Option<WikiSearchQuery>,
}

pub async fn run_worker(
    cmd_rx: Arc<Mutex<mpsc::UnboundedReceiver<NetworkCommand>>>,
    ev_tx: Arc<Mutex<mpsc::UnboundedSender<NetworkEvent>>>,
) {
    let client = reqwest::Client::builder()
        .user_agent("wiki-tui/0.1.0 (https://github.com/sharkthakftw/WikiTUI)") // unique id required or else wikipedia blocks the requests
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    while let Some(cmd) = cmd_rx.lock().await.recv().await {
        let client_ref = client.clone();
        let ev_tx_ref = ev_tx.clone();

        tokio::spawn(async move {
            match cmd {
                NetworkCommand::Search { pane_id, query } => {
                    match search_wikipedia(&client_ref, &query).await {
                        Ok(results) => {
                            let _ = ev_tx_ref.lock().await.send(NetworkEvent::SearchResult {
                                pane_id,
                                query,
                                results,
                            });
                        }
                        Err(err) => {
                            let _ = ev_tx_ref.lock().await.send(NetworkEvent::Error {
                                pane_id,
                                message: err,
                            });
                        }
                    }
                }
                NetworkCommand::FetchArticle { pane_id, title } => {
                    match fetch_article_wikipedia(&client_ref, &title).await {
                        Ok(content) => {
                            let _ = ev_tx_ref.lock().await.send(NetworkEvent::ArticleResult {
                                pane_id,
                                title,
                                content,
                            });
                        }
                        Err(err) => {
                            let _ = ev_tx_ref.lock().await.send(NetworkEvent::Error {
                                pane_id,
                                message: err,
                            });
                        }
                    }
                }
            }
        });
    }
}

async fn search_wikipedia(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<SearchResultItem>, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", query),
            ("utf8", "1"),
            ("format", "json"),
            ("srlimit", "50"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let search_resp: WikiSearchResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

    let mut items = Vec::new();
    if let Some(q) = search_resp.query {
        for item in q.search {
            let clean_snippet = item
                .snippet
                .replace("<span class=\"searchmatch\">", "")
                .replace("</span>", "")
                .replace("&quot;", "\"")
                .replace("&amp;", "&");
            items.push(SearchResultItem {
                title: item.title,
                snippet: clean_snippet,
            });
        }
    }

    Ok(items)
}

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

async fn fetch_article_wikipedia(client: &reqwest::Client, title: &str) -> Result<String, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "parse"),
            ("page", title),
            ("prop", "text"),
            ("format", "json"),
            ("disableeditsection", "1"),
            ("disabletoc", "1"),
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
