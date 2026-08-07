use crate::api::{NetworkCommand, NetworkEvent, SearchResultItem};
use crate::layout::{LayoutNode, SplitDirection};
use crate::parser::{parse_wikipedia_html, ParsedDocument};
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
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

#[derive(Clone, Debug)]
pub struct Pane {
    pub id: usize,
    pub content: PaneContent,
    pub selected_idx: usize,
    pub scroll_offset: usize,
    pub is_loading: bool,
}

impl Pane {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            content: PaneContent::Empty,
            selected_idx: 0,
            scroll_offset: 0,
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
        self.search_input.clear();
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
        self.exit_search_mode();

        if !query.is_empty() {
            let active_pane = self.active_pane_mut();
            let pane_id = active_pane.id;
            active_pane.is_loading = true;
            active_pane.selected_idx = 0;
            active_pane.scroll_offset = 0;

            let _ = self.cmd_tx.send(NetworkCommand::Search { pane_id, query });
        }
    }

    // navigation inside active pane (search results / article text)
    pub fn select_next_item(&mut self) {
        let pane = self.active_pane_mut();
        match &pane.content {
            PaneContent::SearchResults { items, .. } => {
                if !items.is_empty() {
                    pane.selected_idx = (pane.selected_idx + 1).min(items.len() - 1);
                }
            }
            PaneContent::ArticleText { parsed_doc, .. } => {
                let lines_count = parsed_doc.lines.len();
                if pane.scroll_offset + 1 < lines_count {
                    pane.scroll_offset += 1;
                }
            }
            _ => {}
        }
    }

    pub fn select_prev_item(&mut self) {
        let pane = self.active_pane_mut();
        match &pane.content {
            PaneContent::SearchResults { .. } if pane.selected_idx > 0 => {
                pane.selected_idx -= 1;
            }
            PaneContent::ArticleText { .. } if pane.scroll_offset > 0 => {
                pane.scroll_offset -= 1;
            }
            _ => {}
        }
    }

    pub fn activate_selected(&mut self) {
        let (pane_id, selected_title) = {
            let pane = self.active_pane();
            if let PaneContent::SearchResults { items, .. } = &pane.content {
                if let Some(item) = items.get(pane.selected_idx) {
                    (pane.id, Some(item.title.clone()))
                } else {
                    (pane.id, None)
                }
            } else {
                (pane.id, None)
            }
        };

        if let Some(title) = selected_title {
            let active_pane = self.active_pane_mut();
            active_pane.is_loading = true;
            let _ = self
                .cmd_tx
                .send(NetworkCommand::FetchArticle { pane_id, title });
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
                    pane.content = PaneContent::ArticleText {
                        title,
                        raw_html: content,
                        parsed_doc,
                        last_width: initial_width,
                    };
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
