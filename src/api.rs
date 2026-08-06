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

#[derive(Deserialize)]
struct WikiPageExtract {
    _title: Option<String>,
    extract: Option<String>,
}

#[derive(Deserialize)]
struct WikiQueryPages {
    pages: std::collections::HashMap<String, WikiPageExtract>,
}

#[derive(Deserialize)]
struct WikiExtractResponse {
    query: Option<WikiQueryPages>,
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
            ("srlimit", "15"),
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

async fn fetch_article_wikipedia(client: &reqwest::Client, title: &str) -> Result<String, String> {
    let url = "https://en.wikipedia.org/w/api.php";
    let res = client
        .get(url)
        .query(&[
            ("action", "query"),
            ("prop", "extracts"),
            ("explaintext", "1"),
            ("titles", title),
            ("format", "json"),
            ("exintro", "0"),
        ])
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;

    let extract_resp: WikiExtractResponse = res
        .json()
        .await
        .map_err(|e| format!("parse error: {}", e))?;

    if let Some(q) = extract_resp.query {
        for (_page_id, page) in q.pages {
            if let Some(extract) = page.extract.filter(|e| !e.trim().is_empty()) {
                return Ok(extract);
            }
        }
    }

    Err("article text not found".to_string())
}
