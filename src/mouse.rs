use crate::app::{App, InputMode, PaneContent, SettingItem};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent, term_width: u16, term_height: u16) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            handle_scroll(app, -1, term_height);
        }
        MouseEventKind::ScrollDown => {
            handle_scroll(app, 1, term_height);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            handle_left_click(app, mouse.column, mouse.row, term_width, term_height);
        }
        _ => {}
    }
}

fn handle_left_click(app: &mut App, col: u16, row: u16, term_width: u16, term_height: u16) {
    let size = Rect::new(0, 0, term_width, term_height);

    if app.feed.active {
        if let Some(item) = app.feed.current_item().cloned() {
            app.feed.active = false;
            app.open_article(&item.title);
        }
        return;
    }

    if app.active_pane().toc_focused {
        let container_rect = if app.zen_mode {
            crate::ui::modals::centered_rect(80, 90, size)
        } else {
            Rect::new(0, 1, term_width, term_height.saturating_sub(2))
        };
        let toc_area = crate::ui::modals::centered_rect(60, 60, container_rect);
        if col > toc_area.x
            && col < toc_area.x + toc_area.width.saturating_sub(1)
            && row > toc_area.y
            && row < toc_area.y + toc_area.height.saturating_sub(1)
        {
            let row_offset = (row - (toc_area.y + 1)) as usize;
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let current_scroll = pane.scroll_offset;
                let active_heading_idx = parsed_doc
                    .headings
                    .iter()
                    .rposition(|h| h.line_idx <= current_scroll)
                    .unwrap_or(0);
                let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);
                let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
                let toc_scroll = selected_idx.saturating_sub(visible_rows / 2);
                let clicked_idx = toc_scroll + row_offset;

                if clicked_idx < parsed_doc.headings.len() {
                    pane.selected_toc_idx = Some(clicked_idx);
                    app.activate_toc_selection(term_height);
                }
            }
        }
        return;
    }

    if app.input_mode == InputMode::SavedListsViewer {
        let area = crate::ui::modals::centered_rect(70, 70, size);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        let left_area = chunks[0];
        let right_area = chunks[1];

        if col > left_area.x
            && col < left_area.x + left_area.width.saturating_sub(1)
            && row > left_area.y
            && row < left_area.y + left_area.height.saturating_sub(1)
        {
            let clicked_list_idx = (row - (left_area.y + 1)) as usize;
            if clicked_list_idx < app.saved_lists.lists.len() {
                app.lists_modal.viewer_list_idx = clicked_list_idx;
                app.lists_modal.viewer_article_idx = 0;
                app.lists_modal.viewer_focus_right = false;
            }
            return;
        }

        if col > right_area.x
            && col < right_area.x + right_area.width.saturating_sub(1)
            && row > right_area.y
            && row < right_area.y + right_area.height.saturating_sub(1)
        {
            let clicked_art_idx = (row - (right_area.y + 1)) as usize;
            if let Some(list) = app.saved_lists.lists.get(app.lists_modal.viewer_list_idx) {
                if clicked_art_idx < list.articles.len() {
                    app.lists_modal.viewer_article_idx = clicked_art_idx;
                    app.lists_modal.viewer_focus_right = true;
                    let title = list.articles[clicked_art_idx].clone();
                    app.input_mode = InputMode::Normal;
                    app.open_article(&title);
                }
            }
            return;
        }
        return;
    }

    if app.input_mode != InputMode::Normal || app.zen_mode {
        return;
    }

    if row == 0 {
        if let Some(tab_idx) = crate::ui::tab_bar::get_tab_at_col(app, term_width, col) {
            app.switch_to_tab(tab_idx);
        }
        return;
    }

    if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        for (pane_idx, rect) in rects {
            if col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
            {
                tab.active_pane_idx = pane_idx;

                let pane = &mut tab.panes[pane_idx];
                match &pane.content {
                    PaneContent::SearchResults { items, .. } => {
                        let inner_y = rect.y + 1;
                        if row >= inner_y && row < rect.y + rect.height.saturating_sub(1) {
                            let row_in_pane = (row - inner_y) as usize;
                            let clicked_line = pane.scroll_offset + row_in_pane;
                            let inner_width = (rect.width as usize).saturating_sub(4);
                            let wrap_w = inner_width.saturating_sub(4).max(10);
                            let mut cur_line = 0;

                            for (i, item) in items.iter().enumerate() {
                                let snippet_lines = if !item.snippet.is_empty() {
                                    crate::ui::pane_view::wrap_text(
                                        &item.snippet.to_lowercase(),
                                        wrap_w,
                                    )
                                    .len()
                                } else {
                                    0
                                };
                                let item_height = 1 + snippet_lines + 1;
                                if clicked_line >= cur_line && clicked_line < cur_line + item_height
                                {
                                    pane.selected_idx = i;
                                    let title = item.title.clone();
                                    app.open_article(&title);
                                    break;
                                }
                                cur_line += item_height;
                            }
                        }
                    }
                    PaneContent::Empty => {
                        let recent_articles = app.get_continue_reading_articles();
                        let inner_height = (rect.height as usize).saturating_sub(2);
                        let show_recent = !recent_articles.is_empty()
                            && inner_height >= (crate::ui::launch_screen::LOGO.len() + 8);

                        if show_recent {
                            let displayed_count = recent_articles.len().min(7);
                            let total_content_height =
                                crate::ui::launch_screen::LOGO.len() + 4 + displayed_count + 2;
                            let v_pad = inner_height.saturating_sub(total_content_height) / 2;
                            let start_row = rect.y
                                + 1
                                + (v_pad as u16)
                                + (crate::ui::launch_screen::LOGO.len() as u16)
                                + 6;

                            if row >= start_row && row < start_row + (displayed_count as u16) {
                                let idx = (row - start_row) as usize;
                                if idx < recent_articles.len() {
                                    let title = recent_articles[idx].clone();
                                    app.open_article(&title);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}

fn handle_scroll(app: &mut App, delta: i32, term_height: u16) {
    if app.feed.active {
        if delta < 0 {
            app.feed.prev_post();
        } else {
            app.feed.next_post();
            app.maybe_fetch_feed_batch();
        }
        return;
    }

    if app.active_pane().toc_focused {
        if delta < 0 {
            app.select_prev_toc_item();
        } else {
            app.select_next_toc_item();
        }
        return;
    }

    match &app.input_mode {
        InputMode::Settings => {
            let total = SettingItem::ALL.len();
            if total > 0 {
                if delta < 0 {
                    app.settings_cursor_idx = if app.settings_cursor_idx == 0 {
                        total - 1
                    } else {
                        app.settings_cursor_idx - 1
                    };
                } else {
                    app.settings_cursor_idx = (app.settings_cursor_idx + 1) % total;
                }
            }
        }
        InputMode::SaveToList => {
            let count =
                app.saved_lists.lists.iter().filter(|l| l.id != "liked").count() + 1;
            if count > 0 {
                if delta < 0 {
                    app.lists_modal.save_cursor_idx = if app.lists_modal.save_cursor_idx == 0 {
                        count - 1
                    } else {
                        app.lists_modal.save_cursor_idx - 1
                    };
                } else {
                    app.lists_modal.save_cursor_idx =
                        (app.lists_modal.save_cursor_idx + 1) % count;
                }
            }
        }
        InputMode::SavedListsViewer => {
            let lists_count = app.saved_lists.lists.len();
            let current_articles_count = app
                .saved_lists
                .lists
                .get(app.lists_modal.viewer_list_idx)
                .map(|l| l.articles.len())
                .unwrap_or(0);

            if app.lists_modal.viewer_focus_right {
                if current_articles_count > 0 {
                    if delta < 0 {
                        app.lists_modal.viewer_article_idx =
                            app.lists_modal.viewer_article_idx.saturating_sub(1);
                    } else {
                        app.lists_modal.viewer_article_idx =
                            (app.lists_modal.viewer_article_idx + 1).min(current_articles_count - 1);
                    }
                }
            } else if lists_count > 0 {
                if delta < 0 {
                    app.lists_modal.viewer_list_idx =
                        app.lists_modal.viewer_list_idx.saturating_sub(1);
                } else {
                    app.lists_modal.viewer_list_idx =
                        (app.lists_modal.viewer_list_idx + 1).min(lists_count - 1);
                }
                app.lists_modal.viewer_article_idx = 0;
            }
        }
        InputMode::CategoryOnboarding => {
            let total = crate::feed::profile::POPULAR_CATEGORIES.len();
            if total > 0 {
                if delta < 0 {
                    app.onboarding.cursor_idx = if app.onboarding.cursor_idx == 0 {
                        total - 1
                    } else {
                        app.onboarding.cursor_idx - 1
                    };
                } else {
                    app.onboarding.cursor_idx = (app.onboarding.cursor_idx + 1) % total;
                }
            }
        }
        InputMode::Normal | InputMode::LocalSearch => {
            if delta < 0 {
                app.scroll_up_lines(3, term_height);
            } else {
                app.scroll_down_lines(3, term_height);
            }
        }
        _ => {}
    }
}
