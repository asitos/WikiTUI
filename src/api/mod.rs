pub mod article;
pub mod feed;
pub mod random;
pub mod search;

use crate::feed::algorithm::FeedItem;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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

pub async fn run_worker(
    cmd_rx: Arc<Mutex<mpsc::UnboundedReceiver<NetworkCommand>>>,
    ev_tx: Arc<Mutex<mpsc::UnboundedSender<NetworkEvent>>>,
) {
    let client = reqwest::Client::builder()
        .user_agent("wikid/1.2.0 (https://github.com/sharkthakftw/wikid)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    while let Some(cmd) = cmd_rx.lock().await.recv().await {
        let client_ref = client.clone();
        let ev_tx_ref = ev_tx.clone();

        tokio::spawn(async move {
            match cmd {
                NetworkCommand::Search { pane_id, query } => {
                    match search::search_wikipedia(&client_ref, &query).await {
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
                    match article::fetch_article_wikipedia(&client_ref, &title).await {
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
                    match random::fetch_random_article(&client_ref).await {
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
                    if let Ok(items) = feed::fetch_feed_batch(&client_ref).await {
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
