use crate::app::{App, InputMode, SettingItem};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

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
    if app.zen_mode || app.feed.active {
        return;
    }

    if app.input_mode != InputMode::Normal {
        return;
    }

    if row == 0 {
        if let Some(tab_idx) = crate::ui::tab_bar::get_tab_at_col(app, term_width, col) {
            app.switch_to_tab(tab_idx);
        }
        return;
    }

    if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = ratatui::layout::Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        for (pane_idx, rect) in rects {
            if col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
            {
                tab.active_pane_idx = pane_idx;
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
