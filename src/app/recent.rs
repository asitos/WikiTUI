use super::App;
use std::collections::HashSet;
use std::path::PathBuf;

impl App {
    pub fn recent_articles_file_path() -> PathBuf {
        crate::paths::config_dir().join("recent_articles.json")
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
        let trimmed = title.trim();
        if trimmed.is_empty() || trimmed.to_lowercase().starts_with("category:") {
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
        let filtered_recent: Vec<String> = self
            .recent_articles
            .iter()
            .filter(|t| !t.to_lowercase().starts_with("category:"))
            .cloned()
            .collect();

        if !filtered_recent.is_empty() {
            return filtered_recent;
        }

        let mut seen = HashSet::new();
        let mut list = Vec::with_capacity(10);
        for l in &self.saved_lists.lists {
            for a in l.articles.iter().rev() {
                if !a.to_lowercase().starts_with("category:") && seen.insert(a.as_str()) {
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
