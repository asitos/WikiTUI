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
        parsed_doc: Box<ParsedDocument>,
        last_width: usize,
        last_show_footnotes: bool,
        last_show_external_links: bool,
        last_heading_marker: bool,
        last_code_line_numbers: bool,
        last_show_icons: bool,
        last_show_images: bool,
        last_max_image_height: usize,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalMatch {
    pub line_idx: usize,
    pub span_idx: usize,
    pub char_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSelection {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl TextSelection {
    pub fn normalized(&self) -> ((usize, usize), (usize, usize)) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn contains_line(&self, line_idx: usize) -> bool {
        let (start, end) = self.normalized();
        line_idx >= start.0 && line_idx <= end.0
    }
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
    pub text_selection: Option<TextSelection>,
    pub is_mouse_selecting: bool,
    pub selection_anchor: Option<(usize, usize)>,
    pub is_loading: bool,
    pub loading_title: Option<String>,
    pub show_toc: bool,
    pub selected_toc_idx: Option<usize>,
    pub toc_focused: bool,
    pub loaded_images: std::collections::HashMap<String, std::path::PathBuf>,
    pub halfblock_cache:
        std::collections::HashMap<(String, usize, usize), Vec<ratatui::text::Line<'static>>>,

    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
    pub intra_jump_back: Vec<usize>,
    pub intra_jump_forward: Vec<usize>,
    pub current_request_id: u64,
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
            text_selection: None,
            is_mouse_selecting: false,
            selection_anchor: None,
            is_loading: false,
            loading_title: None,
            show_toc: false,
            selected_toc_idx: None,
            toc_focused: false,
            loaded_images: std::collections::HashMap::new(),
            halfblock_cache: std::collections::HashMap::new(),

            history_back: Vec::new(),
            history_forward: Vec::new(),
            intra_jump_back: Vec::new(),
            intra_jump_forward: Vec::new(),
            current_request_id: 0,
        }
    }

    pub fn prepare_for_article_fetch(&mut self, title: &str) {
        self.is_loading = true;
        self.loading_title = Some(title.to_string());
        self.selected_link_idx = None;
        self.text_selection = None;
        self.is_mouse_selecting = false;
        self.selection_anchor = None;
        self.intra_jump_back.clear();
        self.intra_jump_forward.clear();
        self.scroll_offset = 0;
        self.show_toc = false;
        self.selected_toc_idx = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ensure_parsed_width(
        &mut self,
        width: usize,
        show_footnotes: bool,
        show_external_links: bool,
        heading_marker: bool,
        code_line_numbers: bool,
        show_icons: bool,
        show_images: bool,
        max_image_height: usize,
    ) {
        if let PaneContent::ArticleText {
            raw_html,
            parsed_doc,
            last_width,
            last_show_footnotes,
            last_show_external_links,
            last_heading_marker,
            last_code_line_numbers,
            last_show_icons,
            last_show_images,
            last_max_image_height,
            ..
        } = &mut self.content
        {
            if *last_width == width
                && *last_show_footnotes == show_footnotes
                && *last_show_external_links == show_external_links
                && *last_heading_marker == heading_marker
                && *last_code_line_numbers == code_line_numbers
                && *last_show_icons == show_icons
                && *last_show_images == show_images
                && *last_max_image_height == max_image_height
            {
                return;
            }
            **parsed_doc = parse_wikipedia_html(
                raw_html,
                width,
                show_footnotes,
                show_external_links,
                heading_marker,
                code_line_numbers,
                show_icons,
                show_images,
                max_image_height,
            );
            *last_width = width;
            *last_show_footnotes = show_footnotes;
            *last_show_external_links = show_external_links;
            *last_heading_marker = heading_marker;
            *last_code_line_numbers = code_line_numbers;
            *last_show_icons = show_icons;
            *last_show_images = show_images;
            *last_max_image_height = max_image_height;
            if let Some(idx) = self.selected_link_idx {
                if idx >= parsed_doc.links.len() {
                    self.selected_link_idx = if parsed_doc.links.is_empty() {
                        None
                    } else {
                        Some(parsed_doc.links.len() - 1)
                    };
                }
            }
            self.recompute_local_matches();
        }
    }

    pub fn recompute_local_matches(&mut self) {
        self.local_matches.clear();
        let query = self.local_search_query.to_lowercase();
        if query.trim().is_empty() {
            self.selected_match_idx = None;
            return;
        }

        if let PaneContent::ArticleText { parsed_doc, .. } = &self.content {
            for (line_idx, line) in parsed_doc.lines.iter().enumerate() {
                if let Some(full_lower) = parsed_doc.plain_text_lower.get(line_idx) {
                    for (match_pos, _) in full_lower.match_indices(&query) {
                        let mut current_offset = 0;
                        let mut start_span_idx = 0;
                        for (idx, span) in line.spans.iter().enumerate() {
                            let span_len = span.content.len();
                            if current_offset + span_len > match_pos {
                                start_span_idx = idx;
                                break;
                            }
                            current_offset += span_len;
                        }
                        self.local_matches.push(LocalMatch {
                            line_idx,
                            span_idx: start_span_idx,
                            char_offset: match_pos,
                        });
                    }
                }
            }
            if !self.local_matches.is_empty() {
                if let Some(sel) = self.selected_match_idx {
                    self.selected_match_idx = Some(sel.min(self.local_matches.len() - 1));
                } else {
                    self.selected_match_idx = Some(0);
                }
            } else {
                self.selected_match_idx = None;
            }
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

    pub fn effective_viewport_height(&self, term_height: u16) -> usize {
        if self.viewport_height > 0 {
            self.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        }
    }

    pub fn page_scroll_step(&self, term_height: u16) -> usize {
        (self.effective_viewport_height(term_height) * 3 / 4).max(1)
    }

    pub fn max_scroll(&self, term_height: u16) -> usize {
        let viewport = self.effective_viewport_height(term_height);
        match &self.content {
            PaneContent::ArticleText { parsed_doc, .. } => {
                parsed_doc.lines.len().saturating_sub(viewport)
            }
            _ => 0,
        }
    }
}
