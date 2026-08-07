use crate::api::SearchResultItem;
use crate::parser::{ParsedDocument, parse_wikipedia_html};

#[derive(Clone, Debug)]
pub enum PaneContent {
    Empty,
    SearchResults {
        query: String,
        items: Vec<SearchResultItem>,
    },
    ArticleText {
        title: String,
        raw_html: String,
        parsed_doc: ParsedDocument,
        last_width: usize,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalMatch {
    pub line_idx: usize,
    pub span_idx: usize,
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub id: usize,
    pub content: PaneContent,
    pub selected_idx: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub selected_link_idx: Option<usize>,
    pub local_search_query: String,
    pub local_matches: Vec<LocalMatch>,
    pub selected_match_idx: Option<usize>,
    pub is_loading: bool,
    pub show_toc: bool,
    pub selected_toc_idx: Option<usize>,
    pub toc_focused: bool,
}

impl Pane {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            content: PaneContent::Empty,
            selected_idx: 0,
            scroll_offset: 0,
            viewport_height: 0,
            selected_link_idx: None,
            local_search_query: String::new(),
            local_matches: Vec::new(),
            selected_match_idx: None,
            is_loading: false,
            show_toc: false,
            selected_toc_idx: None,
            toc_focused: false,
        }
    }

    pub fn ensure_parsed_width(&mut self, width: usize) {
        if let PaneContent::ArticleText {
            raw_html,
            parsed_doc,
            last_width,
            ..
        } = &mut self.content
        {
            if *last_width == width {
                return;
            }
            *parsed_doc = parse_wikipedia_html(raw_html, width);
            *last_width = width;
        }
    }
}
