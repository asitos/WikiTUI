use super::App;
use std::collections::HashSet;
use std::path::PathBuf;

impl App {
    pub fn recent_articles_file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            dir.push("recent_articles.json");
            dir
        } else {
            PathBuf::from("recent_articles.json")
        }
    }

    pub fn load_recent_articles() -> Vec<String> {
        let path = Self::recent_articles_file_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str::<Vec<String>>(&c).ok())
            .unwrap_or_default()
    }

    pub fn save_recent_articles(&self) {
        let path = Self::recent_articles_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.recent_articles) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn record_recent_article(&mut self, title: &str) {
        if title.trim().is_empty() {
            return;
        }
        self.recent_articles.retain(|t| t != title);
        self.recent_articles.insert(0, title.to_string());
        if self.recent_articles.len() > 10 {
            self.recent_articles.truncate(10);
        }
        self.save_recent_articles();
    }

    pub fn get_continue_reading_articles(&self) -> Vec<String> {
        if !self.recent_articles.is_empty() {
            return self.recent_articles.clone();
        }

        let mut seen = HashSet::new();
        let mut list = Vec::with_capacity(10);
        for l in &self.saved_lists.lists {
            for a in l.articles.iter().rev() {
                if seen.insert(a.as_str()) {
                    list.push(a.clone());
                    if list.len() >= 10 {
                        return list;
                    }
                }
            }
        }
        list
    }
}
