use crate::api::SearchResultItem;
use crate::parser::{parse_wikipedia_html, ParsedDocument};

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
        last_show_footnotes: bool,
        last_show_external_links: bool,
        last_heading_marker: bool,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalMatch {
    pub line_idx: usize,
    pub span_idx: usize,
    pub char_offset: usize,
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

    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
    pub intra_jump_back: Vec<usize>,
    pub intra_jump_forward: Vec<usize>,
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

            history_back: Vec::new(),
            history_forward: Vec::new(),
            intra_jump_back: Vec::new(),
            intra_jump_forward: Vec::new(),
        }
    }

    pub fn ensure_parsed_width(
        &mut self,
        width: usize,
        show_footnotes: bool,
        show_external_links: bool,
        heading_marker: bool,
    ) {
        if let PaneContent::ArticleText {
            raw_html,
            parsed_doc,
            last_width,
            last_show_footnotes,
            last_show_external_links,
            last_heading_marker,
            ..
        } = &mut self.content
        {
            if *last_width == width
                && *last_show_footnotes == show_footnotes
                && *last_show_external_links == show_external_links
                && *last_heading_marker == heading_marker
            {
                return;
            }
            *parsed_doc = parse_wikipedia_html(
                raw_html,
                width,
                show_footnotes,
                show_external_links,
                heading_marker,
            );
            *last_width = width;
            *last_show_footnotes = show_footnotes;
            *last_show_external_links = show_external_links;
            *last_heading_marker = heading_marker;
        }
    }

    pub fn title(&self) -> Option<String> {
        match &self.content {
            PaneContent::ArticleText { title, .. } => Some(title.clone()),
            _ => None,
        }
    }

    pub fn focused_link(&self) -> Option<&crate::parser::types::Link> {
        if let PaneContent::ArticleText { parsed_doc, .. } = &self.content {
            self.selected_link_idx
                .and_then(|idx| parsed_doc.links.get(idx))
        } else {
            None
        }
    }
}
