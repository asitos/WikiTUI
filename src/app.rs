use crate::api::{NetworkCommand, NetworkEvent, SearchResultItem};
use crate::layout::{LayoutNode, SplitDirection};
use crate::parser::{ParsedDocument, parse_wikipedia_html};
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    LocalSearch,
    Help,
}

fn is_article_link(title: &str) -> bool {
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

pub struct Tab {
    pub name: String,
    pub panes: Vec<Pane>,
    pub active_pane_idx: usize,
    pub layout_root: LayoutNode,
}

impl Tab {
    pub fn new(name: String, initial_pane_id: usize) -> Self {
        Self {
            name,
            panes: vec![Pane::new(initial_pane_id)],
            active_pane_idx: 0,
            layout_root: LayoutNode::Leaf(0),
        }
    }
}

pub struct App {
    pub running: bool,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub input_mode: InputMode,
    pub search_input: String,
    pub search_opens_new_tab: bool,
    pub waiting_for_split_cmd: bool,
    next_pane_id: usize,
    cmd_tx: mpsc::UnboundedSender<NetworkCommand>,
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

    // search mode
    pub fn enter_search_mode(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_opens_new_tab = true;
        self.search_input.clear();
    }

    pub fn edit_search_mode(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_opens_new_tab = false;
        let existing_query = match &self.active_pane().content {
            PaneContent::SearchResults { query, .. } => Some(query.clone()),
            _ => None,
        };
        if let Some(query) = existing_query {
            self.search_input = query;
        } else {
            self.search_input.clear();
        }
    }

    pub fn exit_search_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
    }

    pub fn type_search_char(&mut self, c: char) {
        if self.input_mode == InputMode::Search {
            self.search_input.push(c);
        }
    }

    pub fn backspace_search_char(&mut self) {
        if self.input_mode == InputMode::Search {
            self.search_input.pop();
        }
    }

    pub fn submit_search(&mut self) {
        let query = self.search_input.trim().to_string();
        let open_new_tab = self.search_opens_new_tab;
        self.exit_search_mode();

        if !query.is_empty() {
            if open_new_tab {
                let is_empty = matches!(self.active_pane().content, PaneContent::Empty);
                if !is_empty {
                    self.new_tab();
                }
            }
            let active_pane = self.active_pane_mut();
            let pane_id = active_pane.id;
            active_pane.is_loading = true;
            active_pane.selected_idx = 0;
            active_pane.scroll_offset = 0;

            let _ = self.cmd_tx.send(NetworkCommand::Search { pane_id, query });
        }
    }

    fn calc_max_scroll(total_lines: usize, term_height: u16) -> usize {
        let half_screen = (term_height as usize / 2).max(1);
        total_lines.saturating_sub(half_screen)
    }

    fn search_result_line_range(items: &[SearchResultItem], selected_idx: usize) -> (usize, usize) {
        let start = items
            .iter()
            .take(selected_idx)
            .map(|item| 2 + usize::from(!item.snippet.is_empty()))
            .sum();
        let end = start + 2 + usize::from(!items[selected_idx].snippet.is_empty());
        (start, end)
    }

    pub fn clamp_link_selection_to_viewport(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        let PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
            return;
        };
        if parsed_doc.links.is_empty() {
            pane.selected_link_idx = None;
            return;
        }

        let viewport_h = if pane.viewport_height > 0 {
            pane.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        };

        let view_start = pane.scroll_offset;
        let view_end = pane.scroll_offset + viewport_h;

        let is_visible = pane.selected_link_idx.is_some_and(|idx| {
            if let Some(link) = parsed_doc.links.get(idx) {
                link.line_idx >= view_start && link.line_idx < view_end
            } else {
                false
            }
        });

        if !is_visible {
            let first_in_view = parsed_doc
                .links
                .iter()
                .position(|link| link.line_idx >= view_start && link.line_idx < view_end);

            if let Some(idx) = first_in_view {
                pane.selected_link_idx = Some(idx);
            } else {
                let closest = parsed_doc
                    .links
                    .iter()
                    .position(|link| link.line_idx >= view_start)
                    .unwrap_or(parsed_doc.links.len() - 1);
                pane.selected_link_idx = Some(closest);
            }
        }
    }

    pub fn select_next_item(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let max_scroll = Self::calc_max_scroll(parsed_doc.lines.len(), term_height);
                if pane.scroll_offset < max_scroll {
                    pane.scroll_offset += 1;
                }
            }
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            match &pane.content {
                PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                    pane.selected_idx = (pane.selected_idx + 1).min(items.len() - 1);
                    Self::keep_search_selection_visible(pane, term_height);
                }
                _ => {}
            }
        }
    }

    pub fn select_prev_item(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if pane.scroll_offset > 0 {
                pane.scroll_offset -= 1;
            }
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            if pane.selected_idx > 0 {
                pane.selected_idx -= 1;
                Self::keep_search_selection_visible(pane, 0);
            }
        }
    }

    pub fn activate_selected(&mut self) {
        let (pane_id, selected_title) = {
            let pane = self.active_pane();
            match &pane.content {
                PaneContent::SearchResults { items, .. } => {
                    if let Some(item) = items.get(pane.selected_idx) {
                        (pane.id, Some(item.title.clone()))
                    } else {
                        (pane.id, None)
                    }
                }
                PaneContent::ArticleText { parsed_doc, .. } => {
                    if let Some(link_idx) = pane.selected_link_idx {
                        if let Some(link) = parsed_doc.links.get(link_idx) {
                            (pane.id, Some(link.title.clone()))
                        } else {
                            (pane.id, None)
                        }
                    } else {
                        (pane.id, None)
                    }
                }
                _ => (pane.id, None),
            }
        };

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            let active_pane = self.active_pane_mut();
            active_pane.is_loading = true;
            active_pane.selected_link_idx = None;
            let _ = self
                .cmd_tx
                .send(NetworkCommand::FetchArticle { pane_id, title });
        }
    }

    pub fn activate_selected_in_new_tab(&mut self) {
        let selected_title = {
            let pane = self.active_pane();
            match &pane.content {
                PaneContent::SearchResults { items, .. } => {
                    items.get(pane.selected_idx).map(|item| item.title.clone())
                }
                PaneContent::ArticleText { parsed_doc, .. } => pane
                    .selected_link_idx
                    .and_then(|idx| parsed_doc.links.get(idx))
                    .map(|link| link.title.clone()),
                _ => None,
            }
        };

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            self.new_tab();
            let pane_id = self.active_pane().id;
            let active_pane = self.active_pane_mut();
            active_pane.is_loading = true;
            active_pane.selected_link_idx = None;
            let _ = self
                .cmd_tx
                .send(NetworkCommand::FetchArticle { pane_id, title });
        }
    }

    pub fn activate_selected_in_split(&mut self, direction: SplitDirection) {
        let selected_title = {
            let pane = self.active_pane();
            match &pane.content {
                PaneContent::SearchResults { items, .. } => {
                    items.get(pane.selected_idx).map(|item| item.title.clone())
                }
                PaneContent::ArticleText { parsed_doc, .. } => pane
                    .selected_link_idx
                    .and_then(|idx| parsed_doc.links.get(idx))
                    .map(|link| link.title.clone()),
                _ => None,
            }
        };

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            self.split_active_pane(direction);
            let pane_id = self.active_pane().id;
            let active_pane = self.active_pane_mut();
            active_pane.is_loading = true;
            active_pane.selected_link_idx = None;
            let _ = self
                .cmd_tx
                .send(NetworkCommand::FetchArticle { pane_id, title });
        }
    }

    pub fn focus_next_link(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.links.is_empty() {
                return;
            }
            let next_idx = match pane.selected_link_idx {
                Some(idx) => (idx + 1) % parsed_doc.links.len(),
                None => 0,
            };
            pane.selected_link_idx = Some(next_idx);

            let link_line = parsed_doc.links[next_idx].line_idx;
            if link_line < pane.scroll_offset {
                pane.scroll_offset = link_line;
            } else if link_line >= pane.scroll_offset + 10 {
                pane.scroll_offset = link_line.saturating_sub(5);
            }
        }
    }

    pub fn focus_prev_link(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.links.is_empty() {
                return;
            }
            let len = parsed_doc.links.len();
            let prev_idx = match pane.selected_link_idx {
                Some(idx) => {
                    if idx == 0 {
                        len - 1
                    } else {
                        idx - 1
                    }
                }
                None => len - 1,
            };
            pane.selected_link_idx = Some(prev_idx);

            let link_line = parsed_doc.links[prev_idx].line_idx;
            if link_line < pane.scroll_offset {
                pane.scroll_offset = link_line;
            } else if link_line >= pane.scroll_offset + 10 {
                pane.scroll_offset = link_line.saturating_sub(5);
            }
        }
    }

    pub fn enter_local_search_mode(&mut self) {
        self.input_mode = InputMode::LocalSearch;
        let pane = self.active_pane_mut();
        pane.local_search_query.clear();
        pane.local_matches.clear();
        pane.selected_match_idx = None;
    }

    pub fn toggle_help_popup(&mut self) {
        if self.input_mode == InputMode::Help {
            self.input_mode = InputMode::Normal;
        } else {
            self.input_mode = InputMode::Help;
        }
    }

    pub fn update_local_search(&mut self) {
        let pane = self.active_pane_mut();
        pane.local_matches.clear();
        pane.selected_match_idx = None;

        let query = pane.local_search_query.trim().to_lowercase();
        if query.is_empty() {
            return;
        }

        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            for (line_idx, line) in parsed_doc.lines.iter().enumerate() {
                for (span_idx, span) in line.spans.iter().enumerate() {
                    if span.content.to_lowercase().contains(&query) {
                        pane.local_matches.push(LocalMatch { line_idx, span_idx });
                    }
                }
            }
            if !pane.local_matches.is_empty() {
                pane.selected_match_idx = Some(0);
                pane.scroll_offset = pane.local_matches[0].line_idx;
            }
        }
    }

    pub fn next_local_match(&mut self) {
        let pane = self.active_pane_mut();
        if pane.local_matches.is_empty() {
            return;
        }
        let next_idx = match pane.selected_match_idx {
            Some(idx) => (idx + 1) % pane.local_matches.len(),
            None => 0,
        };
        pane.selected_match_idx = Some(next_idx);
        pane.scroll_offset = pane.local_matches[next_idx].line_idx;
    }

    pub fn prev_local_match(&mut self) {
        let pane = self.active_pane_mut();
        if pane.local_matches.is_empty() {
            return;
        }
        let len = pane.local_matches.len();
        let prev_idx = match pane.selected_match_idx {
            Some(idx) => {
                if idx == 0 {
                    len - 1
                } else {
                    idx - 1
                }
            }
            None => len - 1,
        };
        pane.selected_match_idx = Some(prev_idx);
        pane.scroll_offset = pane.local_matches[prev_idx].line_idx;
    }

    pub fn jump_next_heading(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let next_h = parsed_doc
                    .headings
                    .iter()
                    .find(|h| h.line_idx > pane.scroll_offset);
                if let Some(next_h) = next_h {
                    pane.scroll_offset = next_h.line_idx;
                }
            }
            self.clamp_link_selection_to_viewport(term_height);
        }
    }

    pub fn jump_prev_heading(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let prev_h = parsed_doc
                    .headings
                    .iter()
                    .rfind(|h| h.line_idx < pane.scroll_offset);
                if let Some(prev_h) = prev_h {
                    pane.scroll_offset = prev_h.line_idx;
                }
            }
            self.clamp_link_selection_to_viewport(term_height);
        }
    }

    pub fn scroll_page_down(&mut self, term_height: u16) {
        let step = (term_height as usize * 3 / 4).max(1);
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let max_scroll = Self::calc_max_scroll(parsed_doc.lines.len(), term_height);
                pane.scroll_offset = (pane.scroll_offset + step).min(max_scroll);
            }
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            match &pane.content {
                PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                    pane.selected_idx = (pane.selected_idx + step).min(items.len() - 1);
                    Self::keep_search_selection_visible(pane, term_height);
                }
                _ => {}
            }
        }
    }

    pub fn scroll_page_up(&mut self, term_height: u16) {
        let step = (term_height as usize * 3 / 4).max(1);
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            pane.scroll_offset = pane.scroll_offset.saturating_sub(step);
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            if let PaneContent::SearchResults { .. } = &pane.content {
                pane.selected_idx = pane.selected_idx.saturating_sub(step);
                Self::keep_search_selection_visible(pane, term_height);
            }
        }
    }

    pub fn jump_to_top(&mut self) {
        let pane = self.active_pane_mut();
        pane.scroll_offset = 0;
        pane.selected_idx = 0;
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            pane.selected_link_idx = if !parsed_doc.links.is_empty() { Some(0) } else { None };
        }
    }

    pub fn jump_to_bottom(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                pane.scroll_offset = Self::calc_max_scroll(parsed_doc.lines.len(), term_height);
            }
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            match &pane.content {
                PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                    pane.selected_idx = items.len() - 1;
                    Self::keep_search_selection_visible(pane, term_height);
                }
                _ => {}
            }
        }
    }

    fn keep_search_selection_visible(pane: &mut Pane, term_height: u16) {
        let PaneContent::SearchResults { items, .. } = &pane.content else {
            return;
        };
        if items.is_empty() {
            return;
        }

        let (selected_start, selected_end) =
            Self::search_result_line_range(items, pane.selected_idx);
        let viewport_height = if pane.viewport_height > 0 {
            pane.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        };

        if selected_start < pane.scroll_offset {
            pane.scroll_offset = selected_start;
        } else if selected_end > pane.scroll_offset + viewport_height {
            pane.scroll_offset = selected_end - viewport_height;
        }
    }

    // network event handling
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

    fn find_pane_mut(&mut self, target_id: usize) -> Option<&mut Pane> {
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                if pane.id == target_id {
                    return Some(pane);
                }
            }
        }
        None
    }

    // tab management
    pub fn new_tab(&mut self) {
        let name = format!("tab {}", self.tabs.len() + 1);
        self.tabs.push(Tab::new(name, self.next_pane_id));
        self.next_pane_id += 1;
        self.active_tab_idx = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_idx = (self.active_tab_idx + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_tab_idx == 0 {
                self.active_tab_idx = self.tabs.len() - 1;
            } else {
                self.active_tab_idx -= 1;
            }
        }
    }

    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_idx);
            if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            }
        } else {
            let new_pane_id = self.next_pane_id;
            self.next_pane_id += 1;
            self.tabs[0] = Tab::new("home".to_string(), new_pane_id);
            self.active_tab_idx = 0;
        }
    }

    // split management
    pub fn split_active_pane(&mut self, direction: SplitDirection) {
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let tab = self.active_tab_mut();
        let current_pane_idx = tab.active_pane_idx;

        tab.panes.push(Pane::new(new_pane_id));
        let new_pane_idx = tab.panes.len() - 1;

        tab.layout_root
            .split_pane(current_pane_idx, new_pane_idx, direction);
        tab.active_pane_idx = new_pane_idx;
    }

    pub fn close_active_pane(&mut self) {
        if self.active_tab().panes.len() <= 1 {
            self.close_current_tab();
            return;
        }

        let tab = self.active_tab_mut();
        let target_idx = tab.active_pane_idx;

        if let Some(new_root) = tab.layout_root.remove_pane(target_idx) {
            tab.layout_root = new_root;
            tab.layout_root.decrement_indices_above(target_idx);

            tab.panes.remove(target_idx);

            if tab.active_pane_idx >= tab.panes.len() {
                tab.active_pane_idx = tab.panes.len() - 1;
            }
        }
    }

    pub fn navigate_panes(&mut self, dir: char, term_width: u16, term_height: u16) {
        use crate::layout::find_pane_in_direction;
        use ratatui::layout::Rect;

        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = self.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        if let Some(next_idx) = find_pane_in_direction(&rects, tab.active_pane_idx, dir) {
            tab.active_pane_idx = next_idx;
        }
    }
}
