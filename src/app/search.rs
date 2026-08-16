use crate::api::{NetworkCommand, SearchResultItem};
use crate::app::pane::{LocalMatch, Pane, PaneContent};
use crate::app::App;

impl App {
    pub fn enter_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Search;
        self.search_opens_new_tab = true;
        self.search_input.clear();
        self.search_cursor_pos = 0;
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
        self.search_cursor_pos = self.search_input.chars().count();
    }

    pub fn exit_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Normal;
        self.search_input.clear();
        self.search_cursor_pos = 0;
    }

    pub fn fetch_random_article(&mut self) {
        if self.active_pane().is_loading {
            return;
        }

        let is_empty = matches!(self.active_pane().content, PaneContent::Empty);
        let pane_id = if is_empty {
            self.active_pane().id
        } else {
            let next_id = self.next_pane_id;
            self.next_pane_id += 1;
            let tab_name = "loading...".to_string();
            self.tabs.push(crate::app::tab::Tab::new(tab_name, next_id));
            self.active_tab_idx = self.tabs.len() - 1;
            next_id
        };

        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.is_loading = true;
        }

        let _ = self
            .cmd_tx
            .send(NetworkCommand::FetchRandomArticle { pane_id });
    }

    pub fn type_search_char(&mut self, c: char) {
        if self.input_mode == crate::app::InputMode::Search {
            let char_count = self.search_input.chars().count();
            if self.search_cursor_pos >= char_count {
                self.search_input.push(c);
            } else {
                let mut chars: Vec<char> = self.search_input.chars().collect();
                chars.insert(self.search_cursor_pos, c);
                self.search_input = chars.into_iter().collect();
            }
            self.search_cursor_pos += 1;
        }
    }

    pub fn backspace_search_char(&mut self) {
        if self.input_mode == crate::app::InputMode::Search && self.search_cursor_pos > 0 {
            let mut chars: Vec<char> = self.search_input.chars().collect();
            chars.remove(self.search_cursor_pos - 1);
            self.search_input = chars.into_iter().collect();
            self.search_cursor_pos -= 1;
        }
    }

    pub fn delete_word_left(&mut self) {
        if self.input_mode == crate::app::InputMode::Search && self.search_cursor_pos > 0 {
            let mut chars: Vec<char> = self.search_input.chars().collect();
            let mut end_idx = self.search_cursor_pos;

            while end_idx > 0 && chars[end_idx - 1].is_whitespace() {
                end_idx -= 1;
            }
            while end_idx > 0 && !chars[end_idx - 1].is_whitespace() {
                end_idx -= 1;
            }

            chars.drain(end_idx..self.search_cursor_pos);
            self.search_input = chars.into_iter().collect();
            self.search_cursor_pos = end_idx;
        }
    }

    pub fn delete_search_char(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
            let char_count = self.search_input.chars().count();
            if self.search_cursor_pos < char_count {
                let mut chars: Vec<char> = self.search_input.chars().collect();
                chars.remove(self.search_cursor_pos);
                self.search_input = chars.into_iter().collect();
            }
        }
    }

    pub fn move_search_cursor_left(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
            self.search_cursor_pos = self.search_cursor_pos.saturating_sub(1);
        }
    }

    pub fn move_search_cursor_right(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
            let char_count = self.search_input.chars().count();
            if self.search_cursor_pos < char_count {
                self.search_cursor_pos += 1;
            }
        }
    }

    pub fn move_search_cursor_home(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
            self.search_cursor_pos = 0;
        }
    }

    pub fn move_search_cursor_end(&mut self) {
        if self.input_mode == crate::app::InputMode::Search {
            self.search_cursor_pos = self.search_input.chars().count();
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

        let end = if let Some(item) = items.get(selected_idx) {
            start + 2 + usize::from(!item.snippet.is_empty())
        } else {
            start
        };
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

    pub(crate) fn keep_local_match_visible(pane: &mut crate::app::pane::Pane, term_height: u16) {
        let target_line = match (pane.selected_match_idx, &pane.local_matches) {
            (Some(idx), matches) if !matches.is_empty() => matches[idx].line_idx,
            _ => return,
        };

        let viewport_height = if pane.viewport_height > 0 {
            pane.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        };

        let margin = 8.min(viewport_height.saturating_sub(1) / 2);

        let top_threshold = pane.scroll_offset.saturating_add(margin);
        let bottom_threshold = (pane.scroll_offset + viewport_height).saturating_sub(margin);

        if target_line < top_threshold {
            pane.scroll_offset = target_line.saturating_sub(margin);
        } else if target_line >= bottom_threshold {
            pane.scroll_offset = (target_line + margin + 1).saturating_sub(viewport_height);
        }
    }

    fn sync_link_focus_to_current_match(pane: &mut Pane) {
        let Some(match_idx) = pane.selected_match_idx else {
            return;
        };
        let Some(m) = pane.local_matches.get(match_idx) else {
            return;
        };
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if let Some(link_idx) = parsed_doc.links.iter().position(|link| {
                link.span_indices
                    .iter()
                    .any(|&(l, s)| l == m.line_idx && s == m.span_idx)
            }) {
                pane.selected_link_idx = Some(link_idx);
            }
        }
    }

    pub fn update_local_search(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        pane.local_matches.clear();
        pane.selected_match_idx = None;

        let query = pane.local_search_query.to_lowercase();
        if query.trim().is_empty() {
            return;
        }

        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            for (line_idx, line) in parsed_doc.lines.iter().enumerate() {
                let full_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
                let full_lower = full_text.to_lowercase();
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
                    pane.local_matches.push(LocalMatch {
                        line_idx,
                        span_idx: start_span_idx,
                    });
                }
            }
            if !pane.local_matches.is_empty() {
                pane.selected_match_idx = Some(0);
                Self::keep_local_match_visible(pane, term_height);
                Self::sync_link_focus_to_current_match(pane);
            }
        }
    }

    pub fn next_local_match(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if pane.local_matches.is_empty() {
            return;
        }
        let next_idx = match pane.selected_match_idx {
            Some(idx) => (idx + 1) % pane.local_matches.len(),
            None => 0,
        };
        pane.selected_match_idx = Some(next_idx);
        Self::keep_local_match_visible(pane, term_height);
        Self::sync_link_focus_to_current_match(pane);
    }

    pub fn prev_local_match(&mut self, term_height: u16) {
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
        Self::keep_local_match_visible(pane, term_height);
        Self::sync_link_focus_to_current_match(pane);
    }
}
