use crate::api::{NetworkCommand, SearchResultItem};
use crate::app::pane::{LocalMatch, PaneContent};
use crate::app::App;

impl App {
    pub fn enter_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Search;
        self.search_opens_new_tab = true;
        self.search_input.clear();
    }

    pub fn edit_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Search;
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
        self.input_mode = crate::app::InputMode::Normal;
        self.search_input.clear();
    }

    pub fn type_search_char(&mut self, c: char) {
        if self.input_mode == crate::app::InputMode::Search {
            self.search_input.push(c);
        }
    }

    pub fn backspace_search_char(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
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

    pub(crate) fn search_result_line_range(
        items: &[SearchResultItem],
        selected_idx: usize,
    ) -> (usize, usize) {
        let start = items
            .iter()
            .take(selected_idx)
            .map(|item| 2 + usize::from(!item.snippet.is_empty()))
            .sum();
        let end = start + 2 + usize::from(!items[selected_idx].snippet.is_empty());
        (start, end)
    }

    pub(crate) fn keep_search_selection_visible(
        pane: &mut crate::app::pane::Pane,
        term_height: u16,
    ) {
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

    pub fn enter_local_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::LocalSearch;
        let pane = self.active_pane_mut();
        pane.local_search_query.clear();
        pane.local_matches.clear();
        pane.selected_match_idx = None;
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
}
