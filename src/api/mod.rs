pub mod article;
pub mod feed;
pub mod random;
pub mod search;
pub mod stats;

pub use stats::WikiStatistics;

use crate::feed::algorithm::FeedItem;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub title: String,
    pub snippet: String,
}

pub enum NetworkCommand {
    Search {
        pane_id: usize,
        query: String,
        limit: usize,
        timeout: u64,
    },
    FetchArticle {
        pane_id: usize,
        title: String,
        timeout: u64,
        offline_cache: bool,
        cache_lifetime: u64,
    },
    FetchRandomArticle {
        pane_id: usize,
        timeout: u64,
        offline_cache: bool,
        cache_lifetime: u64,
    },
    FetchFeedBatch {
        timeout: u64,
    },
    FetchStats {
        timeout: u64,
    },
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
    StatsLoaded(WikiStatistics),
    Error {
        pane_id: usize,
        message: String,
    },
}

pub fn run_worker(cmd_rx: Receiver<NetworkCommand>, ev_tx: Sender<NetworkEvent>) {
    let agent: std::sync::Arc<ureq::Agent> = std::sync::Arc::new(
        ureq::builder()
            .user_agent(concat!(
                "wikid/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/sharkthakftw/wikid)"
            ))
            .build(),
    );

    while let Ok(cmd) = cmd_rx.recv() {
        let agent = agent.clone();
        let ev_tx = ev_tx.clone();

        std::thread::spawn(move || match cmd {
            NetworkCommand::Search {
                pane_id,
                query,
                limit,
                timeout,
            } => match search::search_wikipedia(&agent, &query, limit, timeout) {
                Ok(results) => {
                    let _ = ev_tx.send(NetworkEvent::SearchResult {
                        pane_id,
                        query,
                        results,
                    });
                }
                Err(err) => {
                    let _ = ev_tx.send(NetworkEvent::Error {
                        pane_id,
                        message: err,
                    });
                }
            },
            NetworkCommand::FetchArticle {
                pane_id,
                title,
                timeout,
                offline_cache,
                cache_lifetime,
            } => {
                match article::fetch_article_wikipedia(
                    &agent,
                    &title,
                    timeout,
                    offline_cache,
                    cache_lifetime,
                ) {
                    Ok(content) => {
                        let _ = ev_tx.send(NetworkEvent::ArticleResult {
                            pane_id,
                            title,
                            content,
                        });
                    }
                    Err(err) => {
                        let _ = ev_tx.send(NetworkEvent::Error {
                            pane_id,
                            message: err,
                        });
                    }
                }
            }
            NetworkCommand::FetchRandomArticle {
                pane_id,
                timeout,
                offline_cache,
                cache_lifetime,
            } => {
                match random::fetch_random_article(&agent, timeout, offline_cache, cache_lifetime) {
                    Ok((title, content)) => {
                        let _ = ev_tx.send(NetworkEvent::ArticleResult {
                            pane_id,
                            title,
                            content,
                        });
                    }
                    Err(err) => {
                        let _ = ev_tx.send(NetworkEvent::Error {
                            pane_id,
                            message: err,
                        });
                    }
                }
            }
            NetworkCommand::FetchFeedBatch { timeout } => {
                if let Ok(items) = feed::fetch_feed_batch(&agent, timeout) {
                    let _ = ev_tx.send(NetworkEvent::FeedBatchLoaded { items });
                }
            }
            NetworkCommand::FetchStats { timeout } => {
                if let Ok(statistics) = stats::fetch_wiki_statistics(&agent, timeout) {
                    let _ = ev_tx.send(NetworkEvent::StatsLoaded(statistics));
                }
            }
        });
    }
}
