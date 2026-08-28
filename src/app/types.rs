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

pub fn is_article_link(title: &str) -> bool {
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
