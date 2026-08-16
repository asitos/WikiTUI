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
                link.span_indices
                    .iter()
                    .any(|(l, _)| *l >= view_start && *l < view_end)
            } else {
                false
            }
        });

        if !is_visible {
            let first_in_view = parsed_doc.links.iter().position(|link| {
                link.span_indices
                    .iter()
                    .any(|(l, _)| *l >= view_start && *l < view_end)
            });

            if let Some(idx) = first_in_view {
                pane.selected_link_idx = Some(idx);
            } else {
                let closest = parsed_doc
                    .links
                    .iter()
                    .position(|link| link.span_indices.iter().any(|(l, _)| *l >= view_start))
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
                Self::keep_search_selection_visible(pane, term_height);
            }
        }
    }

    pub fn activate_selected(&mut self, term_height: u16) {
        let (_pane_id, selected_title) = {
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

        if let Some(target) = selected_title {
            if let Some(anchor) = target.strip_prefix('#') {
                let pane = self.active_pane_mut();
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    if let Some(&target_line) = parsed_doc.reference_targets.get(anchor) {
                        let current_scroll = pane.scroll_offset;
                        pane.jump_stack.push(current_scroll);
                        pane.scroll_offset = target_line;

                        if let Some(target_link_idx) = parsed_doc.links.iter().position(|l| {
                            l.span_indices.iter().any(|(line, _)| *line == target_line)
                        }) {
                            pane.selected_link_idx = Some(target_link_idx);
                        }

                        self.clamp_link_selection_to_viewport(term_height);
                        self.set_status_message(if anchor.starts_with("cite_note") {
                            "jumped to reference (press H to return)"
                        } else {
                            "jumped to citation (press H to return)"
                        });
                    }
                }
            } else if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("//")
            {
                crate::clipboard::copy_to_clipboard(&target);
                self.set_status_message(format!("copied external link: {}", target));
            } else if is_article_link(&target) {
                self.open_article(&target);
            }
        }
    }

    pub fn activate_search_result_digit(&mut self, digit: char) {
        let idx = if digit == '0' { 9 } else { (digit as usize) - ('1' as usize) };
        let pane = self.active_pane_mut();
        let target_title = if let PaneContent::SearchResults { items, .. } = &mut pane.content {
            if idx < items.len() {
                pane.selected_idx = idx;
                Some(items[idx].title.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(title) = target_title {
            self.open_article(&title);
        }
    }

    pub fn open_article(&mut self, title: &str) {
        let current_title = self.active_pane().title();
        let pane_id = self.active_pane().id;
        let active_pane = self.active_pane_mut();
        if let Some(old_title) = current_title {
            if old_title != title {
                active_pane.history_back.push(old_title);
                active_pane.history_forward.clear();
            }
        }
        active_pane.is_loading = true;
        active_pane.selected_link_idx = None;
        active_pane.jump_stack.clear();
        let _ = self.cmd_tx.send(NetworkCommand::FetchArticle {
            pane_id,
            title: title.to_string(),
        });
    }

    pub fn history_back(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if let Some(prev_scroll) = pane.jump_stack.pop() {
            pane.scroll_offset = prev_scroll;
            self.clamp_link_selection_to_viewport(term_height);
            return;
        }

        let current_title = self.active_pane().title();
        let active_pane = self.active_pane_mut();
        if let Some(target_title) = active_pane.history_back.pop() {
            if let Some(cur) = current_title {
                active_pane.history_forward.push(cur);
            }
            let pane_id = active_pane.id;
            active_pane.is_loading = true;
            active_pane.selected_link_idx = None;
            active_pane.jump_stack.clear();
            let _ = self.cmd_tx.send(NetworkCommand::FetchArticle {
                pane_id,
                title: target_title,
            });
        }
    }

    pub fn history_forward(&mut self) {
        let current_title = self.active_pane().title();
        let active_pane = self.active_pane_mut();
        if let Some(target_title) = active_pane.history_forward.pop() {
            if let Some(cur) = current_title {
                active_pane.history_back.push(cur);
            }
            let pane_id = active_pane.id;
            active_pane.is_loading = true;
            active_pane.selected_link_idx = None;
            let _ = self.cmd_tx.send(NetworkCommand::FetchArticle {
                pane_id,
                title: target_title,
            });
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

            let link_line = parsed_doc.links[next_idx]
                .span_indices
                .first()
                .map_or(0, |(l, _)| *l);
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

            let link_line = parsed_doc.links[prev_idx]
                .span_indices
                .first()
                .map_or(0, |(l, _)| *l);
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
            pane.selected_link_idx = if !parsed_doc.links.is_empty() {
                Some(0)
            } else {
                None
            };
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

    pub fn set_status_message(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn copy_focused_link(&mut self) {
        let pane = self.active_pane();
        let target_url = match &pane.content {
            PaneContent::ArticleText {
                title, parsed_doc, ..
            } => {
                if let Some(idx) = pane.selected_link_idx {
                    if let Some(link) = parsed_doc.links.get(idx) {
                        if link.title.starts_with("http://")
                            || link.title.starts_with("https://")
                            || link.title.starts_with("//")
                        {
                            link.title.clone()
                        } else {
                            format!(
                                "https://en.wikipedia.org/wiki/{}",
                                link.title.replace(' ', "_")
                            )
                        }
                    } else {
                        format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
                    }
                } else {
                    format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
                }
            }
            PaneContent::SearchResults { items, .. } => {
                if let Some(item) = items.get(pane.selected_idx) {
                    format!(
                        "https://en.wikipedia.org/wiki/{}",
                        item.title.replace(' ', "_")
                    )
                } else {
                    return;
                }
            }
            _ => return,
        };

        crate::clipboard::copy_to_clipboard(&target_url);
        self.set_status_message(format!("copied: {}", target_url));
    }

    pub fn copy_article_link(&mut self) {
        let pane = self.active_pane();
        let target_url = match &pane.content {
            PaneContent::ArticleText { title, .. } => {
                format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
            }
            PaneContent::SearchResults { query, .. } => {
                format!(
                    "https://en.wikipedia.org/wiki/Special:Search?search={}",
                    query.replace(' ', "_")
                )
            }
            _ => return,
        };

        crate::clipboard::copy_to_clipboard(&target_url);
        self.set_status_message(format!("copied article: {}", target_url));
    }
}
