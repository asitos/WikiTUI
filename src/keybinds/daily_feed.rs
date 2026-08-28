use crate::app::{App, InputMode};
use crate::ui::modals::get_feed_entries;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_daily_feed_mode(app: &mut App, key: KeyEvent) {
    let state = match &app.daily_feed_modal {
        Some(s) => s.clone(),
        None => {
            app.input_mode = InputMode::Normal;
            return;
        }
    };

    let entries = get_feed_entries(app, state.kind);
    let total = entries.len();

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_daily_feed_modal();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if total > 0 {
                if let Some(modal) = &mut app.daily_feed_modal {
                    if modal.cursor_idx + 1 < total {
                        modal.cursor_idx += 1;
                    }
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(modal) = &mut app.daily_feed_modal {
                if modal.cursor_idx > 0 {
                    modal.cursor_idx -= 1;
                }
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            if let Some(modal) = &mut app.daily_feed_modal {
                modal.cursor_idx = 0;
            }
        }
        KeyCode::Char('G') | KeyCode::End => {
            if total > 0 {
                if let Some(modal) = &mut app.daily_feed_modal {
                    modal.cursor_idx = total.saturating_sub(1);
                }
            }
        }
        KeyCode::Enter => {
            if let Some(entry) = entries.get(state.cursor_idx) {
                let target = entry.target_article.clone();
                app.close_daily_feed_modal();
                app.open_article(&target);
            }
        }
        KeyCode::Char('t') => {
            if let Some(entry) = entries.get(state.cursor_idx) {
                let target = entry.target_article.clone();
                app.close_daily_feed_modal();
                if !matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                    app.new_tab();
                }
                app.open_article(&target);
            }
        }
        _ => {}
    }
}