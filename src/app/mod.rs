pub mod layout_mgr;
pub mod navigation;
pub mod pane;
pub mod search;
pub mod tab;

pub use pane::{LocalMatch, Pane, PaneContent};
pub use tab::Tab;

use crate::api::{NetworkCommand, NetworkEvent};
use crate::parser::parse_wikipedia_html;
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    LocalSearch,
    Help,
}

pub(crate) fn is_article_link(title: &str) -> bool {
    let lower = title.to_lowercase();
    !lower.starts_with("http://")
        && !lower.starts_with("https://")
        && !lower.ends_with(".jpg")
        && !lower.ends_with(".png")
        && !lower.ends_with(".svg")
        && !lower.ends_with(".gif")
        && !lower.ends_with(".jpeg")
        && !lower.ends_with(".webp")
}

pub struct App {
    pub running: bool,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub input_mode: InputMode,
    pub search_input: String,
    pub search_opens_new_tab: bool,
    pub waiting_for_split_cmd: bool,
    pub(crate) next_pane_id: usize,
    pub(crate) cmd_tx: mpsc::UnboundedSender<NetworkCommand>,
}

impl App {
    pub fn new(cmd_tx: mpsc::UnboundedSender<NetworkCommand>) -> Self {
        let mut app = Self {
            running: true,
            tabs: Vec::new(),
            active_tab_idx: 0,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            search_opens_new_tab: true,
            waiting_for_split_cmd: false,
            next_pane_id: 1,
            cmd_tx,
        };
        app.tabs.push(Tab::new("home".to_string(), 0));
        app
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_idx]
    }

    pub fn active_pane(&self) -> &Pane {
        &self.active_tab().panes[self.active_tab().active_pane_idx]
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        let tab = self.active_tab_mut();
        let idx = tab.active_pane_idx;
        &mut tab.panes[idx]
    }

    pub fn toggle_help_popup(&mut self) {
        if self.input_mode == InputMode::Help {
            self.input_mode = InputMode::Normal;
        } else {
            self.input_mode = InputMode::Help;
        }
    }

    pub fn handle_network_event(&mut self, ev: NetworkEvent) {
        match ev {
            NetworkEvent::SearchResult {
                pane_id,
                query,
                results,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    pane.is_loading = false;
                    pane.selected_idx = 0;
                    pane.scroll_offset = 0;
                    pane.content = PaneContent::SearchResults {
                        query,
                        items: results,
                    };
                }
            }
            NetworkEvent::ArticleResult {
                pane_id,
                title,
                content,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    pane.is_loading = false;
                    pane.scroll_offset = 0;
                    let initial_width = 80;
                    let parsed_doc = parse_wikipedia_html(&content, initial_width);
                    let initial_link_idx = if !parsed_doc.links.is_empty() { Some(0) } else { None };
                    pane.content = PaneContent::ArticleText {
                        title,
                        raw_html: content,
                        parsed_doc,
                        last_width: initial_width,
                    };
                    pane.selected_link_idx = initial_link_idx;
                }
            }
            NetworkEvent::Error { pane_id, message } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    pane.is_loading = false;
                    pane.content = PaneContent::Error(message);
                }
            }
        }
    }
}
