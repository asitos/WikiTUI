use crate::app::{App, InputMode, PaneContent};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_categories_mode(app: &mut App, key: KeyEvent) {
    let categories_count = match &app.active_pane().content {
        PaneContent::ArticleText { parsed_doc, .. } => parsed_doc.categories.len(),
        _ => 0,
    };

    match key.code {
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            if categories_count > 0 {
                app.categories_modal.cursor_idx = (app.categories_modal.cursor_idx + 1) % categories_count;
            }
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            if categories_count > 0 {
                if app.categories_modal.cursor_idx == 0 {
                    app.categories_modal.cursor_idx = categories_count.saturating_sub(1);
                } else {
                    app.categories_modal.cursor_idx -= 1;
                }
            }
        }
        KeyCode::Enter => {
            let target_title =
                if let PaneContent::ArticleText { parsed_doc, .. } = &app.active_pane().content {
                    parsed_doc
                        .categories
                        .get(app.categories_modal.cursor_idx)
                        .map(|cat| format!("Category:{}", cat))
                } else {
                    None
                };

            if let Some(cat_title) = target_title {
                app.input_mode = InputMode::Normal;
                app.open_article(&cat_title);
            }
        }
        KeyCode::Char('y') => {
            let copy_url =
                if let PaneContent::ArticleText { parsed_doc, .. } = &app.active_pane().content {
                    parsed_doc
                        .categories
                        .get(app.categories_modal.cursor_idx)
                        .map(|cat| {
                            format!(
                                "https://en.wikipedia.org/wiki/Category:{}",
                                cat.replace(' ', "_")
                            )
                        })
                } else {
                    None
                };

            if let Some(url) = copy_url {
                crate::clipboard::copy_to_clipboard(&url);
                app.set_status_message(format!("copied category: {}", url));
            }
        }
        KeyCode::Char('c') | KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}
