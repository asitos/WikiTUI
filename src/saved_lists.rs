use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedArticle {
    pub title: String,
    pub snippet: Option<String>,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedList {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub articles: Vec<SavedArticle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedListsStore {
    pub version: u32,
    pub lists: Vec<SavedList>,
}

impl Default for SavedListsStore {
    fn default() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        Self {
            version: 1,
            lists: vec![SavedList {
                id: "read_later".to_string(),
                name: "Read Later".to_string(),
                description: "Default saved articles".to_string(),
                created_at: now,
                articles: Vec::new(),
            }],
        }
    }
}

impl SavedListsStore {
    pub fn file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            let _ = fs::create_dir_all(&dir);
            dir.push("saved_articles.json");
            dir
        } else {
            PathBuf::from("saved_articles.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        let loaded = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<SavedListsStore>(&content).ok());

        if let Some(store) = loaded {
            return store;
        }
        let store = Self::default();
        store.save();
        store
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(pretty_json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, pretty_json);
        }
    }

    pub fn create_list(&mut self, name: &str, description: &str) -> String {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return String::new();
        }
        let id = clean_name.to_lowercase().replace(' ', "_");

        if !self.lists.iter().any(|l| l.id == id) {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
            self.lists.push(SavedList {
                id: id.clone(),
                name: clean_name.to_string(),
                description: description.trim().to_string(),
                created_at: now,
                articles: Vec::new(),
            });
            self.save();
        }
        id
    }

    pub fn delete_list(&mut self, list_id: &str) {
        self.lists.retain(|l| l.id != list_id);
        self.save();
    }

    pub fn toggle_article_in_list(
        &mut self,
        list_id: &str,
        title: &str,
        snippet: Option<&str>,
    ) -> bool {
        let title_trimmed = title.trim();
        if title_trimmed.is_empty() {
            return false;
        }

        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            if let Some(idx) = list.articles.iter().position(|a| a.title == title_trimmed) {
                list.articles.remove(idx);
                self.save();
                return false;
            } else {
                let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
                list.articles.push(SavedArticle {
                    title: title_trimmed.to_string(),
                    snippet: snippet.map(|s| s.trim().to_string()),
                    saved_at: now,
                });
                self.save();
                return true;
            }
        }
        false
    }

    pub fn is_article_in_list(&self, list_id: &str, title: &str) -> bool {
        let title_trimmed = title.trim();
        if let Some(list) = self.lists.iter().find(|l| l.id == list_id) {
            list.articles.iter().any(|a| a.title == title_trimmed)
        } else {
            false
        }
    }
}
