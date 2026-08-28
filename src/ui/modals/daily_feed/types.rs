#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DailyFeedKind {
    News,
    OnThisDay,
    MostRead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OnThisDayTab {
    #[default]
    Events,
    Births,
    Deaths,
    Holidays,
}

#[derive(Clone, Debug)]
pub struct DailyFeedModalState {
    pub kind: DailyFeedKind,
    pub cursor_idx: usize,
    pub link_idx: usize,
    pub otd_tab: OnThisDayTab,
}

impl Default for DailyFeedModalState {
    fn default() -> Self {
        Self {
            kind: DailyFeedKind::News,
            cursor_idx: 0,
            link_idx: 0,
            otd_tab: OnThisDayTab::Events,
        }
    }
}

pub struct FeedEntry {
    pub title: String,
    pub target_article: String,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStyle {
    Normal,
    Bold,
    Italic,
    Link { link_idx: usize, title: String },
    BoldLink { link_idx: usize, title: String },
}

#[derive(Debug, Clone)]
pub struct StyledChunk {
    pub text: String,
    pub style: SpanStyle,
}
