pub mod article;
pub mod feed;
pub mod random;
pub mod search;

use crate::feed::algorithm::FeedItem;
use std::sync::mpsc::{Receiver, Sender};

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

pub fn run_worker(cmd_rx: Receiver<NetworkCommand>, ev_tx: Sender<NetworkEvent>) {
    let agent: ureq::Agent = ureq::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!(
            "wikid/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/sharkthakftw/wikid)"
        ))
        .build();

    while let Ok(cmd) = cmd_rx.recv() {
        let agent = agent.clone();
        let ev_tx = ev_tx.clone();

        std::thread::spawn(move || match cmd {
            NetworkCommand::Search { pane_id, query } => {
                match search::search_wikipedia(&agent, &query) {
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
                }
            }
            NetworkCommand::FetchArticle { pane_id, title } => {
                match article::fetch_article_wikipedia(&agent, &title) {
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
            NetworkCommand::FetchRandomArticle { pane_id } => {
                match random::fetch_random_article(&agent) {
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
            NetworkCommand::FetchFeedBatch => {
                if let Ok(items) = feed::fetch_feed_batch(&agent) {
                    let _ = ev_tx.send(NetworkEvent::FeedBatchLoaded { items });
                }
            }
        });
    }
}
