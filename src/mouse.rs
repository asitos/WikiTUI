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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(size);
        let inner_area = chunks[0];
        let card_area = crate::ui::modals::centered_rect(80, 85, inner_area);

        if row == card_area.y + card_area.height.saturating_sub(1)
            && col >= card_area.x + card_area.width.saturating_sub(14)
        {
            app.toggle_feed_like();
        } else if let Some(item) = app.feed.current_item().cloned() {
            app.feed.active = false;
            app.open_article(&item.title);
        }
        return;
    }

    if app.input_mode == InputMode::Settings {
        let area = crate::ui::modals::centered_rect(50, 75, size);
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );

        if col >= inner.x
            && col < inner.x + inner.width
            && row >= inner.y
            && row < inner.y + inner.height
        {
            if let Some((idx, item, val_start_x)) =
                crate::ui::modals::settings::get_setting_row_at(inner, row)
            {
                app.settings_cursor_idx = idx;
                let is_numeric = matches!(
                    item,
                    SettingItem::ScrollLines
                        | SettingItem::SearchLimit
                        | SettingItem::NetworkTimeout
                        | SettingItem::CacheLifetime
                );
                if is_numeric {
                    if col >= val_start_x {
                        let rel_col = col - val_start_x;
                        if rel_col <= 3 {
                            app.adjust_selected_setting(-1);
                        } else if rel_col >= 11 {
                            app.adjust_selected_setting(1);
                        } else {
                            app.adjust_selected_setting(0);
                        }
                    }
                } else {
                    app.adjust_selected_setting(0);
                }
            }
        }
        return;
    }

    if app.input_mode == InputMode::CategoryOnboarding {
        let area = crate::ui::modals::centered_rect(60, 80, size);
        if col > area.x && col < area.x + area.width.saturating_sub(1) {
            match crate::ui::modals::onboarding::get_onboarding_row_at(area, row) {
                Some(crate::ui::modals::onboarding::OnboardingHit::Category(idx)) => {
                    app.onboarding.cursor_idx = idx;
                    if let Some(val) = app.onboarding.selected.get_mut(idx) {
                        *val = !*val;
                    }
                }
                Some(crate::ui::modals::onboarding::OnboardingHit::Submit) => {
                    app.submit_category_onboarding();
                }
                None => {}
            }
        }
        return;
    }

    if app.input_mode == InputMode::SaveToList {
        let area = crate::ui::modals::centered_rect(55, 60, size);
        if col > area.x && col < area.x + area.width.saturating_sub(1) {
            match crate::ui::modals::lists::get_save_to_list_item_at(app, area, row) {
                Some(crate::ui::modals::lists::SaveToListHit::Toggle(idx)) => {
                    app.lists_modal.save_cursor_idx = idx;
                    let custom_lists: Vec<_> = app
                        .saved_lists
                        .lists
                        .iter()
                        .filter(|l| l.id != "liked")
                        .cloned()
                        .collect();
                    if let Some(list) = custom_lists.get(idx) {
                        let list_id = list.id.clone();
                        let target_title = app.lists_modal.target_title.clone();
                        app.saved_lists
                            .toggle_article_in_list(&list_id, &target_title);
                    }
                }
                Some(crate::ui::modals::lists::SaveToListHit::CreateNew) => {
                    let custom_lists_count = app
                        .saved_lists
                        .lists
                        .iter()
                        .filter(|l| l.id != "liked")
                        .count();
                    app.lists_modal.save_cursor_idx = custom_lists_count;
                    app.lists_modal.create_input.clear();
                    app.lists_modal.create_return_mode = InputMode::SaveToList;
                    app.input_mode = InputMode::CreateNewList;
                }
                None => {}
            }
        }
        return;
    }

    if app.input_mode == InputMode::Confirm {
        let area = crate::ui::modals::centered_rect(50, 30, size);
        if let Some(c) = crate::ui::modals::lists::get_confirm_button_at(app, area, col, row) {
            crate::keybinds::confirm::handle_confirm_mode(
                app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
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
        if col > toc_area.x && col < toc_area.x + toc_area.width.saturating_sub(1) {
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let current_scroll = pane.scroll_offset;
                let active_heading_idx = parsed_doc
                    .headings
                    .iter()
                    .rposition(|h| h.line_idx <= current_scroll)
                    .unwrap_or(0);
                let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

                if let Some(clicked_idx) =
                    crate::ui::modals::toc::get_toc_heading_at(parsed_doc, selected_idx, toc_area, row)
                {
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
            if let Some(clicked_list_idx) =
                crate::ui::modals::lists::get_saved_lists_viewer_item_at(app, false, left_area, row)
            {
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
            if let Some(clicked_art_idx) =
                crate::ui::modals::lists::get_saved_lists_viewer_item_at(app, true, right_area, row)
            {
                if let Some(list) = app.saved_lists.lists.get(app.lists_modal.viewer_list_idx) {
                    if clicked_art_idx < list.articles.len() {
                        app.lists_modal.viewer_article_idx = clicked_art_idx;
                        app.lists_modal.viewer_focus_right = true;
                        let title = list.articles[clicked_art_idx].clone();
                        app.input_mode = InputMode::Normal;
                        app.open_article(&title);
                    }
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
                            if let Some(item_idx) =
                                crate::ui::pane_view::get_search_result_at_line(
                                    items,
                                    pane.selected_idx,
                                    inner_width,
                                    clicked_line,
                                )
                            {
                                pane.selected_idx = item_idx;
                                let title = items[item_idx].title.clone();
                                app.open_article(&title);
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
