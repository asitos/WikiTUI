pub mod layout_mgr;
pub mod navigation;
pub mod pane;
pub mod search;
pub mod tab;

pub use pane::{LocalMatch, Pane, PaneContent};
pub use tab::Tab;

use crate::api::{NetworkCommand, NetworkEvent};
use crate::parser::parse_wikipedia_html;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction {
    DeleteList { list_id: String, title: String },
    DeleteArticle { list_id: String, title: String },
    ResetFeed,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingItem {
    LikedReadonly,
    AutoRestoreSession,
    ConfirmQuit,
    RoundedBorders,
    Icons,
    ScrollIndicator,
    HeadingMarker,
    ScrollLines,
    UnderlineLinks,
    ShowFootnotes,
    ShowExternalLinks,
    TocSectionNumbers,
    CodeLineNumbers,
    SearchLimit,
    NetworkTimeout,
}

impl SettingItem {
    pub const ALL: &'static [SettingItem] = &[
        SettingItem::LikedReadonly,
        SettingItem::AutoRestoreSession,
        SettingItem::ConfirmQuit,
        SettingItem::RoundedBorders,
        SettingItem::Icons,
        SettingItem::ScrollIndicator,
        SettingItem::HeadingMarker,
        SettingItem::ScrollLines,
        SettingItem::UnderlineLinks,
        SettingItem::ShowFootnotes,
        SettingItem::ShowExternalLinks,
        SettingItem::TocSectionNumbers,
        SettingItem::CodeLineNumbers,
        SettingItem::SearchLimit,
        SettingItem::NetworkTimeout,
    ];

    pub fn section(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly
            | SettingItem::AutoRestoreSession
            | SettingItem::ConfirmQuit => "general",
            SettingItem::RoundedBorders | SettingItem::Icons | SettingItem::ScrollIndicator => "ui",
            SettingItem::HeadingMarker
            | SettingItem::ScrollLines
            | SettingItem::UnderlineLinks
            | SettingItem::ShowFootnotes
            | SettingItem::ShowExternalLinks
            | SettingItem::TocSectionNumbers
            | SettingItem::CodeLineNumbers => "reader",
            SettingItem::SearchLimit => "search",
            SettingItem::NetworkTimeout => "network",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly => "liked list read-only",
            SettingItem::AutoRestoreSession => "auto-restore last session",
            SettingItem::ConfirmQuit => "confirm before quitting",
            SettingItem::RoundedBorders => "rounded borders",
            SettingItem::Icons => "icons",
            SettingItem::ScrollIndicator => "scroll indicator",
            SettingItem::HeadingMarker => "heading marker",
            SettingItem::ScrollLines => "scroll lines per step",
            SettingItem::UnderlineLinks => "underline links",
            SettingItem::ShowFootnotes => "show footnotes & citations",
            SettingItem::ShowExternalLinks => "show external links section",
            SettingItem::TocSectionNumbers => "toc section numbers",
            SettingItem::CodeLineNumbers => "code line numbers",
            SettingItem::SearchLimit => "search results limit",
            SettingItem::NetworkTimeout => "request timeout",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly => "prevent manual deletion of articles from liked list",
            SettingItem::AutoRestoreSession => "automatically restore last session on startup",
            SettingItem::ConfirmQuit => "prompt for confirmation when exiting wikid",
            SettingItem::RoundedBorders => "use rounded border corners instead of sharp",
            SettingItem::Icons => "display nerd fonts",
            SettingItem::ScrollIndicator => "display scrollbar track on right edge of content panes",
            SettingItem::HeadingMarker => "display colored bar marker (▍) before section headings",
            SettingItem::ScrollLines => "number of lines to scroll per j/k press (1-20)",
            SettingItem::UnderlineLinks => "display underlined modifier on article links",
            SettingItem::ShowFootnotes => "show inline reference numbers and references section",
            SettingItem::ShowExternalLinks => "show the external links section at the bottom",
            SettingItem::TocSectionNumbers => "display hierarchical numbers in table of contents",
            SettingItem::CodeLineNumbers => "display line numbers in code blocks",
            SettingItem::SearchLimit => "maximum number of search results to fetch (5-50)",
            SettingItem::NetworkTimeout => "network request timeout in seconds (2-60s)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    LocalSearch,
    Help,
    CategoryOnboarding,
    SaveToList,
    CreateNewList,
    SavedListsViewer,
    Confirm,
    Settings,
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

#[derive(Clone, Debug)]
pub struct ClosedTabState {
    pub title: String,
    pub scroll_offset: usize,
    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchModalState {
    pub input: String,
    pub cursor_pos: usize,
    pub opens_new_tab: bool,
}

#[derive(Clone, Debug)]
pub struct OnboardingModalState {
    pub cursor_idx: usize,
    pub selected: Vec<bool>,
}

impl Default for OnboardingModalState {
    fn default() -> Self {
        Self {
            cursor_idx: 0,
            selected: vec![
                false, false, false, false, true, false, false, true, true, false, false, true,
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListsModalState {
    pub target_title: String,
    pub target_snippet: Option<String>,
    pub save_cursor_idx: usize,
    pub create_input: String,
    pub create_return_mode: InputMode,
    pub viewer_list_idx: usize,
    pub viewer_article_idx: usize,
    pub viewer_focus_right: bool,
}

impl Default for ListsModalState {
    fn default() -> Self {
        Self {
            target_title: String::new(),
            target_snippet: None,
            save_cursor_idx: 0,
            create_input: String::new(),
            create_return_mode: InputMode::SaveToList,
            viewer_list_idx: 0,
            viewer_article_idx: 0,
            viewer_focus_right: false,
        }
    }
}

pub struct App {
    pub running: bool,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub input_mode: InputMode,
    pub search_modal: SearchModalState,
    pub waiting_for_split_cmd: bool,
    pub zen_mode: bool,

    pub feed: crate::feed::FeedState,
    pub onboarding: OnboardingModalState,

    pub saved_lists: crate::saved_lists::SavedListsStore,
    pub lists_modal: ListsModalState,
    pub confirm_action: Option<ConfirmAction>,
    pub config: crate::config::Config,
    pub config_last_mtime: Option<std::time::SystemTime>,
    pub last_config_check: std::time::Instant,
    pub settings_cursor_idx: usize,
    pub closed_tabs_stack: Vec<ClosedTabState>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub wiki_stats: crate::api::WikiStatistics,
    pub recent_articles: Vec<String>,
    pub launch_quote_idx: usize,

    pub(crate) next_pane_id: usize,
    pub(crate) cmd_tx: Sender<NetworkCommand>,
}

impl App {
    pub fn recent_articles_file_path() -> std::path::PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut dir = std::path::PathBuf::from(home);
            dir.push(".config");
            dir.push("wikid");
            dir.push("recent_articles.json");
            dir
        } else {
            std::path::PathBuf::from("recent_articles.json")
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

        let mut seen = std::collections::HashSet::new();
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

    pub fn send_fetch_article(&self, pane_id: usize, title: String) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchArticle {
            pane_id,
            title,
            timeout: self.config.network.timeout,
        });
    }

    pub fn send_fetch_random_article(&self, pane_id: usize) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchRandomArticle {
            pane_id,
            timeout: self.config.network.timeout,
        });
    }

    pub fn send_fetch_feed_batch(&self) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchFeedBatch {
            timeout: self.config.network.timeout,
        });
    }

    pub fn send_fetch_stats(&self) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchStats {
            timeout: self.config.network.timeout,
        });
    }

    pub fn new(cmd_tx: Sender<NetworkCommand>) -> Self {
        let config = crate::config::Config::load();
        let _ = cmd_tx.send(NetworkCommand::FetchStats {
            timeout: config.network.timeout,
        });
        let quote_idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        let mut app = Self {
            running: true,
            tabs: Vec::new(),
            active_tab_idx: 0,
            input_mode: InputMode::Normal,
            search_modal: SearchModalState::default(),
            waiting_for_split_cmd: false,
            zen_mode: false,
            feed: crate::feed::FeedState::new(),
            onboarding: OnboardingModalState::default(),

            saved_lists: crate::saved_lists::SavedListsStore::load(),
            lists_modal: ListsModalState::default(),
            confirm_action: None,
            config,
            config_last_mtime: crate::config::Config::get_modified_time(),
            last_config_check: std::time::Instant::now(),
            settings_cursor_idx: 0,
            closed_tabs_stack: Vec::new(),
            status_message: None,
            wiki_stats: crate::api::WikiStatistics::default(),
            recent_articles: Self::load_recent_articles(),
            launch_quote_idx: quote_idx,

            next_pane_id: 1,
            cmd_tx,
        };
        for title in &app.feed.profile.liked_articles {
            app.saved_lists
                .set_article_in_list("liked", "Liked", title, true);
        }
        if let Some(liked_list) = app.saved_lists.lists.iter().find(|l| l.id == "liked") {
            for title in &liked_list.articles {
                app.feed.profile.liked_articles.insert(title.clone());
            }
        }
        if app.config.general.auto_restore_session {
            if let Some(session) = crate::session::SessionState::load() {
                app.restore_session(session);
            }
        }
        if app.tabs.is_empty() {
            app.tabs.push(Tab::new("home".to_string(), 0));
        }
        app
    }

    pub fn check_config_sync(&mut self) {
        if self.last_config_check.elapsed() >= std::time::Duration::from_millis(500) {
            self.last_config_check = std::time::Instant::now();
            self.config.reload_if_changed(&mut self.config_last_mtime);
        }
    }

    pub fn save_session(&self) {
        crate::session::SessionState::save_app_session(self);
    }

    pub fn restore_session(&mut self, state: crate::session::SessionState) {
        state.restore_to_app(self);
    }

    pub fn quit(&mut self) {
        if self.config.general.confirm_quit {
            self.confirm_action = Some(ConfirmAction::Quit);
            self.input_mode = InputMode::Confirm;
        } else {
            self.save_session();
            self.running = false;
        }
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
        self.lists_modal.target_title = title;
        self.lists_modal.target_snippet = snippet;
        self.lists_modal.save_cursor_idx = 0;
        self.input_mode = InputMode::SaveToList;
    }

    pub fn open_saved_lists_viewer(&mut self) {
        self.saved_lists = crate::saved_lists::SavedListsStore::load();
        self.lists_modal.viewer_list_idx = 0;
        self.lists_modal.viewer_article_idx = 0;
        self.lists_modal.viewer_focus_right = false;
        self.input_mode = InputMode::SavedListsViewer;
    }

    pub fn submit_create_new_list(&mut self) {
        let name = self.lists_modal.create_input.trim().to_string();
        if !name.is_empty() {
            let list_id = self.saved_lists.create_list(&name);
            if !self.lists_modal.target_title.is_empty() {
                self.saved_lists
                    .toggle_article_in_list(&list_id, &self.lists_modal.target_title);
            }
        }
        self.lists_modal.create_input.clear();
        self.input_mode = self.lists_modal.create_return_mode.clone();
    }

    pub fn toggle_zen_mode(&mut self) {
        self.zen_mode = !self.zen_mode;
    }

    pub fn maybe_fetch_feed_batch(&mut self) {
        if !self.feed.is_fetching && self.feed.active_idx + 3 >= self.feed.items.len() {
            self.feed.is_fetching = true;
            self.send_fetch_feed_batch();
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
            .onboarding
            .selected
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

    pub fn reset_feed(&mut self) {
        self.feed.reset();
        if let Some(liked_list) = self.saved_lists.lists.iter_mut().find(|l| l.id == "liked") {
            liked_list.articles.clear();
            self.saved_lists.save();
        }
        self.onboarding.cursor_idx = 0;
        self.onboarding.selected = vec![
            false, false, false, false, true, false, false, true, true, false, false, true,
        ];
        self.input_mode = InputMode::CategoryOnboarding;
        self.set_status_message("feed reset: select initial categories");
    }
    pub fn active_tab(&self) -> &Tab {
        let idx = self.active_tab_idx.min(self.tabs.len().saturating_sub(1));
        &self.tabs[idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        if self.tabs.is_empty() {
            self.tabs.push(Tab::new("home".to_string(), 0));
        }
        if self.active_tab_idx >= self.tabs.len() {
            self.active_tab_idx = self.tabs.len() - 1;
        }
        let idx = self.active_tab_idx;
        &mut self.tabs[idx]
    }

    pub fn active_pane(&self) -> &Pane {
        let tab = self.active_tab();
        let idx = tab.active_pane_idx.min(tab.panes.len().saturating_sub(1));
        &tab.panes[idx]
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        let tab = self.active_tab_mut();
        if tab.panes.is_empty() {
            tab.panes.push(Pane::new(0));
        }
        if tab.active_pane_idx >= tab.panes.len() {
            tab.active_pane_idx = tab.panes.len() - 1;
        }
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
                self.record_recent_article(&title);
                let show_footnotes = self.config.reader.show_footnotes;
                let show_external_links = self.config.reader.show_external_links;
                let heading_marker = self.config.reader.heading_marker;
                let code_line_numbers = self.config.reader.code_line_numbers;
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    pane.is_loading = false;
                    pane.scroll_offset = 0;
                    pane.toc_focused = false;
                    let initial_width = 80;
                    let parsed_doc = parse_wikipedia_html(
                        &content,
                        initial_width,
                        show_footnotes,
                        show_external_links,
                        heading_marker,
                        code_line_numbers,
                    );
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
                        last_show_footnotes: show_footnotes,
                        last_show_external_links: show_external_links,
                        last_heading_marker: heading_marker,
                        last_code_line_numbers: code_line_numbers,
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
                for mut item in items {
                    item.is_liked = self.feed.profile.liked_articles.contains(&item.title)
                        || self.saved_lists.is_article_in_list("liked", &item.title);
                    self.feed.add_item(item);
                }
            }
            NetworkEvent::StatsLoaded(stats) => {
                self.wiki_stats = stats;
            }
        }
    }

    pub fn toggle_feed_like(&mut self) {
        if let Some((title, _snippet, is_liked)) = self.feed.toggle_like() {
            self.saved_lists
                .set_article_in_list("liked", "Liked", &title, is_liked);
        }
    }

    pub fn adjust_selected_setting(&mut self, delta: i32) {
        if let Some(item) = SettingItem::ALL.get(self.settings_cursor_idx).copied() {
            match item {
                SettingItem::ScrollLines => {
                    let cur = self.config.reader.scroll_lines as i32;
                    let new_val = if delta == 0 {
                        if cur >= 20 {
                            1
                        } else {
                            cur + 1
                        }
                    } else {
                        (cur + delta).clamp(1, 20)
                    };
                    self.config.reader.scroll_lines = new_val as usize;
                }
                SettingItem::LikedReadonly => {
                    self.config.general.liked_readonly = !self.config.general.liked_readonly;
                }
                SettingItem::AutoRestoreSession => {
                    self.config.general.auto_restore_session =
                        !self.config.general.auto_restore_session;
                }
                SettingItem::ConfirmQuit => {
                    self.config.general.confirm_quit = !self.config.general.confirm_quit;
                }
                SettingItem::RoundedBorders => {
                    self.config.ui.rounded_borders = !self.config.ui.rounded_borders;
                }
                SettingItem::Icons => {
                    self.config.ui.icons = !self.config.ui.icons;
                }
                SettingItem::ScrollIndicator => {
                    self.config.ui.scroll_indicator = !self.config.ui.scroll_indicator;
                }
                SettingItem::HeadingMarker => {
                    self.config.reader.heading_marker = !self.config.reader.heading_marker;
                }
                SettingItem::UnderlineLinks => {
                    self.config.reader.underline_links = !self.config.reader.underline_links;
                }
                SettingItem::ShowFootnotes => {
                    self.config.reader.show_footnotes = !self.config.reader.show_footnotes;
                }
                SettingItem::ShowExternalLinks => {
                    self.config.reader.show_external_links =
                        !self.config.reader.show_external_links;
                }
                SettingItem::TocSectionNumbers => {
                    self.config.reader.toc_section_numbers =
                        !self.config.reader.toc_section_numbers;
                }
                SettingItem::CodeLineNumbers => {
                    self.config.reader.code_line_numbers = !self.config.reader.code_line_numbers;
                }
                SettingItem::SearchLimit => {
                    let cur = self.config.search.limit as i32;
                    let step = 5;
                    let new_val = if delta == 0 {
                        if cur >= 50 {
                            5
                        } else {
                            cur + step
                        }
                    } else {
                        (cur + delta * step).clamp(5, 50)
                    };
                    self.config.search.limit = new_val as usize;
                }
                SettingItem::NetworkTimeout => {
                    let cur = self.config.network.timeout as i32;
                    let step = 2;
                    let new_val = if delta == 0 {
                        if cur >= 60 {
                            2
                        } else {
                            cur + step
                        }
                    } else {
                        (cur + delta * step).clamp(2, 60)
                    };
                    self.config.network.timeout = new_val as u64;
                }
            }
            self.config.save();
            self.config_last_mtime = crate::config::Config::get_modified_time();
        }
    }

    pub fn reset_selected_setting(&mut self) {
        if let Some(item) = SettingItem::ALL.get(self.settings_cursor_idx).copied() {
            let default_config = crate::config::Config::default();
            match item {
                SettingItem::LikedReadonly => {
                    self.config.general.liked_readonly = default_config.general.liked_readonly;
                }
                SettingItem::AutoRestoreSession => {
                    self.config.general.auto_restore_session =
                        default_config.general.auto_restore_session;
                }
                SettingItem::ConfirmQuit => {
                    self.config.general.confirm_quit = default_config.general.confirm_quit;
                }
                SettingItem::RoundedBorders => {
                    self.config.ui.rounded_borders = default_config.ui.rounded_borders;
                }
                SettingItem::Icons => {
                    self.config.ui.icons = default_config.ui.icons;
                }
                SettingItem::ScrollIndicator => {
                    self.config.ui.scroll_indicator = default_config.ui.scroll_indicator;
                }
                SettingItem::HeadingMarker => {
                    self.config.reader.heading_marker = default_config.reader.heading_marker;
                }
                SettingItem::ScrollLines => {
                    self.config.reader.scroll_lines = default_config.reader.scroll_lines;
                }
                SettingItem::UnderlineLinks => {
                    self.config.reader.underline_links = default_config.reader.underline_links;
                }
                SettingItem::ShowFootnotes => {
                    self.config.reader.show_footnotes = default_config.reader.show_footnotes;
                }
                SettingItem::ShowExternalLinks => {
                    self.config.reader.show_external_links =
                        default_config.reader.show_external_links;
                }
                SettingItem::TocSectionNumbers => {
                    self.config.reader.toc_section_numbers =
                        default_config.reader.toc_section_numbers;
                }
                SettingItem::CodeLineNumbers => {
                    self.config.reader.code_line_numbers = default_config.reader.code_line_numbers;
                }
                SettingItem::SearchLimit => {
                    self.config.search.limit = default_config.search.limit;
                }
                SettingItem::NetworkTimeout => {
                    self.config.network.timeout = default_config.network.timeout;
                }
            }
            self.config.save();
            self.config_last_mtime = crate::config::Config::get_modified_time();
        }
    }

    pub fn reset_all_settings(&mut self) {
        self.config = crate::config::Config::default();
        self.config.save();
        self.config_last_mtime = crate::config::Config::get_modified_time();
    }
}
