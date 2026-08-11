use crate::feed::algorithm::FeedItem;
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
    FetchRandomArticle { pane_id: usize },
    FetchFeedBatch,
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
    FeedBatchLoaded {
        items: Vec<FeedItem>,
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
        .user_agent("wikid/1.0.0 (https://github.com/sharkthakftw/wikid)")
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
                NetworkCommand::FetchRandomArticle { pane_id } => {
                    match fetch_random_article(&client_ref).await {
                        Ok((title, content)) => {
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
                NetworkCommand::FetchFeedBatch => {
                    if let Ok(items) = fetch_feed_batch(&client_ref).await {
                        let _ = ev_tx_ref
                            .lock()
                            .await
                            .send(NetworkEvent::FeedBatchLoaded { items });
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

async fn fetch_random_article(client: &reqwest::Client) -> Result<(String, String), String> {
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

#[derive(Deserialize)]
struct WikiCategoryItem {
    title: String,
}

#[derive(Deserialize)]
struct WikiPageProp {
    title: Option<String>,
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

use rand::Rng;

async fn fetch_feed_batch(client: &reqwest::Client) -> Result<Vec<FeedItem>, String> {
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
                ("prop", "extracts|categories"),
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
                ("prop", "extracts|categories"),
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
