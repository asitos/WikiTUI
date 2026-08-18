use crate::api::NetworkCommand;
use crate::app::{App, Pane, Tab};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedPaneState {
    pub title: Option<String>,
    pub scroll_offset: usize,
    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedTabState {
    pub panes: Vec<SavedPaneState>,
    pub active_pane_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionState {
    pub version: u32,
    pub tabs: Vec<SavedTabState>,
    pub active_tab_idx: usize,
}

impl SessionState {
    pub fn file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            dir.push("session.json");
            dir
        } else {
            PathBuf::from("session.json")
        }
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str::<SessionState>(&content).ok()
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn delete_file() {
        let path = Self::file_path();
        let _ = fs::remove_file(path);
    }

    pub fn save_app_session(app: &App) {
        let mut saved_tabs = Vec::new();
        for tab in &app.tabs {
            let mut saved_panes = Vec::new();
            for pane in &tab.panes {
                if pane.title().is_some() || !pane.history_back.is_empty() {
                    saved_panes.push(SavedPaneState {
                        title: pane.title(),
                        scroll_offset: pane.scroll_offset,
                        history_back: pane.history_back.clone(),
                        history_forward: pane.history_forward.clone(),
                    });
                }
            }
            if !saved_panes.is_empty() {
                let active_idx = tab.active_pane_idx.min(saved_panes.len() - 1);
                saved_tabs.push(SavedTabState {
                    panes: saved_panes,
                    active_pane_idx: active_idx,
                });
            }
        }
        if saved_tabs.is_empty() {
            Self::delete_file();
            return;
        }
        let active_tab_idx = app.active_tab_idx.min(saved_tabs.len() - 1);
        let session = SessionState {
            version: 1,
            tabs: saved_tabs,
            active_tab_idx,
        };
        session.save();
    }

    pub fn restore_to_app(self, app: &mut App) {
        if self.tabs.is_empty() {
            return;
        }
        app.tabs.clear();
        for saved_tab in self.tabs {
            let mut panes = Vec::new();
            for saved_pane in saved_tab.panes {
                let pane_id = app.next_pane_id;
                app.next_pane_id += 1;
                let mut pane = Pane::new(pane_id);
                pane.scroll_offset = saved_pane.scroll_offset;
                pane.history_back = saved_pane.history_back;
                pane.history_forward = saved_pane.history_forward;
                if let Some(title) = saved_pane.title {
                    pane.is_loading = true;
                    let _ = app.cmd_tx.send(NetworkCommand::FetchArticle {
                        pane_id,
                        title: title.clone(),
                    });
                }
                panes.push(pane);
            }
            if panes.is_empty() {
                let pane_id = app.next_pane_id;
                app.next_pane_id += 1;
                panes.push(Pane::new(pane_id));
            }
            let active_idx = saved_tab.active_pane_idx.min(panes.len() - 1);
            let tab_title = panes[active_idx]
                .title()
                .unwrap_or_else(|| "home".to_string());
            app.tabs.push(Tab {
                name: tab_title,
                panes,
                active_pane_idx: active_idx,
                layout_root: crate::layout::LayoutNode::Leaf(0),
            });
        }
        if !app.tabs.is_empty() {
            app.active_tab_idx = self.active_tab_idx.min(app.tabs.len() - 1);
        }
        Self::delete_file();
    }
}
