pub mod audio_ctrl;
pub mod events;
pub mod feed_ctrl;
pub mod history;
pub mod layout_mgr;
pub mod navigation;
pub mod network;
pub mod pane;
pub mod recent;
pub mod search;
pub mod settings;
pub mod tab;
pub mod types;

pub use pane::{LocalMatch, Pane, PaneContent};
pub use settings::SettingItem;
pub use tab::Tab;
pub use types::{
    is_article_link, CategoriesModalState, ClosedTabState, ConfirmAction, InputMode,
    ListsModalState, OnboardingModalState, SearchModalState, SettingsModalState,
};

use crate::api::NetworkCommand;
use std::sync::mpsc::Sender;

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
    pub settings_modal: SettingsModalState,
    pub categories_modal: CategoriesModalState,
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
            settings_modal: SettingsModalState::default(),
            categories_modal: CategoriesModalState::default(),
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

    pub fn toggle_categories_modal(&mut self) {
        if self.input_mode == InputMode::Categories {
            self.input_mode = InputMode::Normal;
            return;
        }

        if matches!(self.active_pane().content, PaneContent::ArticleText { .. }) {
            self.categories_modal.cursor_idx = 0;
            self.input_mode = InputMode::Categories;
        }
    }
}
