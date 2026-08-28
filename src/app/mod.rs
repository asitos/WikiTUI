pub mod history;
pub mod layout_mgr;
pub mod navigation;
pub mod pane;
pub mod recent;
pub mod search;
pub mod settings;
pub mod tab;

pub use pane::{LocalMatch, Pane, PaneContent};
pub use settings::SettingItem;
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
    Categories,
    DailyFeedModal,
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
    pub categories_cursor_idx: usize,
    pub closed_tabs_stack: Vec<ClosedTabState>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub wiki_stats: crate::api::WikiStatistics,
    pub daily_feed: Option<crate::api::DailyFeed>,
    pub daily_feed_modal: Option<crate::ui::modals::DailyFeedModalState>,
    pub recent_articles: Vec<String>,
    pub launch_quote_idx: usize,
    pub scroll_drag: Option<crate::mouse::ScrollDragTarget>,
    pub audio_player: crate::audio::AudioPlayer,

    pub(crate) next_pane_id: usize,
    pub(crate) next_request_id: u64,
    pub(crate) cmd_tx: Sender<NetworkCommand>,
}

impl App {
    pub fn open_daily_feed_modal(&mut self, kind: crate::ui::modals::DailyFeedKind) {
        self.daily_feed_modal = Some(crate::ui::modals::DailyFeedModalState {
            kind,
            cursor_idx: 0,
            link_idx: 0,
        });
        self.input_mode = InputMode::DailyFeedModal;
    }

    pub fn close_daily_feed_modal(&mut self) {
        self.daily_feed_modal = None;
        self.input_mode = InputMode::Normal;
    }
    pub fn next_request_id(&mut self) -> u64 {
        let req_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        req_id
    }

    pub fn send_fetch_article(&mut self, pane_id: usize, title: String) {
        let request_id = self.next_request_id();
        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.current_request_id = request_id;
        }
        let _ = self.cmd_tx.send(NetworkCommand::FetchArticle {
            request_id,
            pane_id,
            title,
            timeout: self.config.network.timeout,
            offline_cache: self.config.network.offline_cache,
            cache_lifetime: self.config.network.cache_lifetime,
        });
    }

    pub fn send_fetch_random_article(&mut self, pane_id: usize) {
        let request_id = self.next_request_id();
        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.current_request_id = request_id;
        }
        let _ = self.cmd_tx.send(NetworkCommand::FetchRandomArticle {
            request_id,
            pane_id,
            timeout: self.config.network.timeout,
            offline_cache: self.config.network.offline_cache,
            cache_lifetime: self.config.network.cache_lifetime,
        });
    }

    pub fn send_fetch_feed_batch(&self) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchFeedBatch {
            timeout: self.config.network.timeout,
        });
    }

    pub fn send_fetch_daily_feed(&self) {
        let _ = self.cmd_tx.send(NetworkCommand::FetchDailyFeed {
            timeout: self.config.network.timeout,
            offline_cache: self.config.network.offline_cache,
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
        let (y, m, d) = crate::api::daily_feed::utc_today();
        let cached_feed = if config.network.offline_cache {
            crate::api::daily_feed::get_cached_daily_feed(y, m, d)
        } else {
            None
        };
        let _ = cmd_tx.send(NetworkCommand::FetchDailyFeed {
            timeout: config.network.timeout,
            offline_cache: config.network.offline_cache,
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
            categories_cursor_idx: 0,
            closed_tabs_stack: Vec::new(),
            status_message: None,
            wiki_stats: crate::api::WikiStatistics::default(),
            daily_feed: cached_feed,
            daily_feed_modal: None,
            recent_articles: Self::load_recent_articles(),
            launch_quote_idx: quote_idx,
            scroll_drag: None,
            audio_player: crate::audio::AudioPlayer::new(),

            next_pane_id: 1,
            next_request_id: 1,
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
                request_id,
                pane_id,
                query,
                results,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    if request_id >= pane.current_request_id {
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
            }
            NetworkEvent::ArticleResult {
                request_id,
                pane_id,
                title,
                content,
            } => {
                let is_current = self
                    .find_pane_mut(pane_id)
                    .is_some_and(|p| request_id >= p.current_request_id);

                if is_current {
                    self.record_recent_article(&title);
                    let show_footnotes = self.config.reader.show_footnotes;
                    let show_external_links = self.config.reader.show_external_links;
                    let heading_marker = self.config.reader.heading_marker;
                    let code_line_numbers = self.config.reader.code_line_numbers;
                    if let Some(pane) = self.find_pane_mut(pane_id) {
                        pane.is_loading = false;
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
                        pane.scroll_offset = pane.scroll_offset.min(parsed_doc.lines.len().saturating_sub(1));
                        let initial_link_idx = if !parsed_doc.links.is_empty() {
                            Some(0)
                        } else {
                            None
                        };
                        pane.content = PaneContent::ArticleText {
                            title,
                            raw_html: content,
                            parsed_doc: Box::new(parsed_doc),
                            last_width: initial_width,
                            last_show_footnotes: show_footnotes,
                            last_show_external_links: show_external_links,
                            last_heading_marker: heading_marker,
                            last_code_line_numbers: code_line_numbers,
                        };
                        pane.selected_link_idx = initial_link_idx;
                    }
                }
            }
            NetworkEvent::Error {
                request_id,
                pane_id,
                message,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    if request_id >= pane.current_request_id {
                        pane.is_loading = false;
                        pane.content = PaneContent::Error(message);
                    }
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
            NetworkEvent::DailyFeedLoaded(feed) => {
                self.daily_feed = Some(feed);
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

    pub fn toggle_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.toggle_pause();
            let msg = match self.audio_player.state {
                crate::audio::PlaybackState::Playing => "audio resumed".to_string(),
                crate::audio::PlaybackState::Paused => "audio paused".to_string(),
                crate::audio::PlaybackState::Stopped => "audio stopped".to_string(),
            };
            self.set_status_message(msg);
            return;
        }

        let track_info = match &self.active_pane().content {
            PaneContent::ArticleText {
                parsed_doc, title, ..
            } => {
                if let Some(spoken) = &parsed_doc.spoken_audio {
                    spoken
                        .tracks
                        .first()
                        .map(|t| (title.clone(), t.url.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((play_title, track_url)) = track_info {
            let success = self.audio_player.play(&play_title, &track_url);
            if success {
                self.set_status_message(format!("playing spoken article: {}", play_title));
            } else if self.audio_player.backend.is_none() {
                self.set_status_message(
                    "no audio backend found (install mpv, ffplay, or cvlc)".to_string(),
                );
            } else {
                self.set_status_message("failed to start audio playback".to_string());
            }
        } else {
            self.set_status_message("no spoken audio available for this article".to_string());
        }
    }

    pub fn stop_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.stop();
            self.set_status_message("audio playback stopped".to_string());
        }
    }

    pub fn toggle_categories_modal(&mut self) {
        if self.input_mode == InputMode::Categories {
            self.input_mode = InputMode::Normal;
            return;
        }

        if matches!(self.active_pane().content, PaneContent::ArticleText { .. }) {
            self.categories_cursor_idx = 0;
            self.input_mode = InputMode::Categories;
        }
    }
}
