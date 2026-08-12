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
    CategoryOnboarding,
    SaveToList,
    CreateNewList,
    SavedListsViewer,
    ConfirmDelete,
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
    pub search_cursor_pos: usize,
    pub search_opens_new_tab: bool,
    pub waiting_for_split_cmd: bool,
    pub zen_mode: bool,

    pub feed: crate::feed::FeedState,
    pub onboarding_cursor_idx: usize,
    pub onboarding_selected: Vec<bool>,

    pub saved_lists: crate::saved_lists::SavedListsStore,
    pub save_modal_target_title: String,
    pub save_modal_target_snippet: Option<String>,
    pub save_modal_cursor_idx: usize,
    pub create_list_input: String,
    pub create_list_return_mode: InputMode,
    pub viewer_list_idx: usize,
    pub viewer_article_idx: usize,
    pub viewer_focus_right: bool,

    pub pending_delete_is_list: bool,
    pub pending_delete_title: String,
    pub pending_delete_list_id: String,

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
            search_cursor_pos: 0,
            search_opens_new_tab: true,
            waiting_for_split_cmd: false,
            zen_mode: false,

            feed: crate::feed::FeedState::new(),
            onboarding_cursor_idx: 0,
            onboarding_selected: vec![false, false, false, false, true, false, false, true, true, false, false, true],

            saved_lists: crate::saved_lists::SavedListsStore::load(),
            save_modal_target_title: String::new(),
            save_modal_target_snippet: None,
            save_modal_cursor_idx: 0,
            create_list_input: String::new(),
            create_list_return_mode: InputMode::SaveToList,
            viewer_list_idx: 0,
            viewer_article_idx: 0,
            viewer_focus_right: false,
            pending_delete_is_list: false,
            pending_delete_title: String::new(),
            pending_delete_list_id: String::new(),

            next_pane_id: 1,
            cmd_tx,
        };
        app.tabs.push(Tab::new("home".to_string(), 0));
        app
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn open_save_to_list_modal(&mut self) {
        let pane = self.active_pane();
        let (title, snippet) = match &pane.content {
            PaneContent::ArticleText { title, .. } => (title.clone(), None),
            PaneContent::SearchResults { items, .. } => {
                if let Some(item) = items.get(pane.selected_idx) {
                    (item.title.clone(), Some(item.snippet.clone()))
                } else {
                    return;
                }
            }
            _ => return,
        };

        if title.trim().is_empty() {
            return;
        }

        self.saved_lists = crate::saved_lists::SavedListsStore::load();
        self.save_modal_target_title = title;
        self.save_modal_target_snippet = snippet;
        self.save_modal_cursor_idx = 0;
        self.input_mode = InputMode::SaveToList;
    }

    pub fn open_saved_lists_viewer(&mut self) {
        self.saved_lists = crate::saved_lists::SavedListsStore::load();
        self.viewer_list_idx = 0;
        self.viewer_article_idx = 0;
        self.viewer_focus_right = false;
        self.input_mode = InputMode::SavedListsViewer;
    }

    pub fn submit_create_new_list(&mut self) {
        let name = self.create_list_input.trim().to_string();
        if !name.is_empty() {
            let list_id = self.saved_lists.create_list(&name, "");
            if !self.save_modal_target_title.is_empty() {
                self.saved_lists.toggle_article_in_list(
                    &list_id,
                    &self.save_modal_target_title,
                    self.save_modal_target_snippet.as_deref(),
                );
            }
        }
        self.create_list_input.clear();
        self.input_mode = self.create_list_return_mode.clone();
    }

    pub fn toggle_zen_mode(&mut self) {
        self.zen_mode = !self.zen_mode;
    }

    pub fn maybe_fetch_feed_batch(&mut self) {
        if !self.feed.is_fetching && self.feed.active_idx + 3 >= self.feed.items.len() {
            self.feed.is_fetching = true;
            let _ = self.cmd_tx.send(NetworkCommand::FetchFeedBatch);
        }
    }

    pub fn toggle_feed_mode(&mut self) {
        let is_active = self.feed.toggle_active();
        if is_active {
            if !self.feed.profile.has_onboarded {
                self.input_mode = InputMode::CategoryOnboarding;
            } else if self.feed.items.is_empty() {
                self.maybe_fetch_feed_batch();
            }
        }
    }

    pub fn submit_category_onboarding(&mut self) {
        let chosen_indices: Vec<usize> = self
            .onboarding_selected
            .iter()
            .enumerate()
            .filter_map(|(idx, &sel)| if sel { Some(idx) } else { None })
            .collect();

        self.feed.profile.complete_onboarding(&chosen_indices);
        self.input_mode = InputMode::Normal;
        if self.feed.items.is_empty() {
            self.maybe_fetch_feed_batch();
        }
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
                    pane.toc_focused = false;
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
                    pane.toc_focused = false;
                    let initial_width = 80;
                    let parsed_doc = parse_wikipedia_html(&content, initial_width);
                    let initial_link_idx = if !parsed_doc.links.is_empty() {
                        Some(0)
                    } else {
                        None
                    };
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
            NetworkEvent::FeedBatchLoaded { items } => {
                self.feed.is_fetching = false;
                for item in items {
                    self.feed.add_item(item);
                }
            }
        }
    }
}
