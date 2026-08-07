use crate::api::NetworkCommand;
use crate::app::pane::PaneContent;
use crate::app::{is_article_link, App};
use crate::layout::SplitDirection;

impl App {
    pub(crate) fn calc_max_scroll(total_lines: usize, term_height: u16) -> usize {
        let half_screen = (term_height as usize / 2).max(1);
        total_lines.saturating_sub(half_screen)
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

    pub fn toggle_toc(&mut self) {
        let pane = self.active_pane_mut();
        let has_headings = match &pane.content {
            PaneContent::ArticleText { parsed_doc, .. } => !parsed_doc.headings.is_empty(),
            _ => false,
        };

        if !has_headings {
            pane.show_toc = false;
            pane.toc_focused = false;
            return;
        }

        pane.show_toc = !pane.show_toc;
        pane.toc_focused = pane.show_toc;

        if pane.show_toc {
            let current_scroll = pane.scroll_offset;
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let active_idx = parsed_doc
                    .headings
                    .iter()
                    .rposition(|h| h.line_idx <= current_scroll)
                    .unwrap_or(0);
                pane.selected_toc_idx = Some(active_idx);
            }
        }
    }

    pub fn select_next_toc_item(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.headings.is_empty() {
                return;
            }
            let len = parsed_doc.headings.len();
            let next_idx = match pane.selected_toc_idx {
                Some(idx) => (idx + 1).min(len - 1),
                None => 0,
            };
            pane.selected_toc_idx = Some(next_idx);
        }
    }

    pub fn select_prev_toc_item(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.headings.is_empty() {
                return;
            }
            let prev_idx = match pane.selected_toc_idx {
                Some(idx) => idx.saturating_sub(1),
                None => 0,
            };
            pane.selected_toc_idx = Some(prev_idx);
        }
    }

    pub fn activate_toc_selection(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        let target_line = match (&pane.content, pane.selected_toc_idx) {
            (PaneContent::ArticleText { parsed_doc, .. }, Some(idx)) => {
                parsed_doc.headings.get(idx).map(|h| h.line_idx)
            }
            _ => None,
        };
        if let Some(line) = target_line {
            pane.scroll_offset = line;
        }
        pane.show_toc = false;
        pane.toc_focused = false;
        self.clamp_link_selection_to_viewport(term_height);
    }
}
