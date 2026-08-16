use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_save_to_list_mode(app: &mut App, key: KeyEvent) {
    let custom_lists: Vec<_> = app
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .cloned()
        .collect();
    let total = custom_lists.len() + 1;

    match key.code {
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            app.save_modal_cursor_idx = (app.save_modal_cursor_idx + 1) % total;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            if app.save_modal_cursor_idx == 0 {
                app.save_modal_cursor_idx = total.saturating_sub(1);
            } else {
                app.save_modal_cursor_idx -= 1;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if app.save_modal_cursor_idx < custom_lists.len() {
                let list_id = custom_lists[app.save_modal_cursor_idx].id.clone();
                let target_title = app.save_modal_target_title.clone();
                app.saved_lists
                    .toggle_article_in_list(&list_id, &target_title);
            } else {
                app.create_list_input.clear();
                app.create_list_return_mode = InputMode::SaveToList;
                app.input_mode = InputMode::CreateNewList;
            }
        }
        KeyCode::Char('n') => {
            app.create_list_input.clear();
            app.create_list_return_mode = InputMode::SaveToList;
            app.input_mode = InputMode::CreateNewList;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

pub fn handle_create_new_list_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.create_list_input.push(c);
        }
        KeyCode::Backspace => {
            app.create_list_input.pop();
        }
        KeyCode::Enter => {
            app.submit_create_new_list();
        }
        KeyCode::Esc => {
            app.input_mode = app.create_list_return_mode.clone();
        }
        _ => {}
    }
}

pub fn handle_saved_lists_viewer_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            app.viewer_focus_right = false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.viewer_focus_right = true;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.viewer_focus_right {
                if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                    if !list.articles.is_empty() {
                        app.viewer_article_idx = (app.viewer_article_idx + 1) % list.articles.len();
                    }
                }
            } else if !app.saved_lists.lists.is_empty() {
                app.viewer_list_idx = (app.viewer_list_idx + 1) % app.saved_lists.lists.len();
                app.viewer_article_idx = 0;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.viewer_focus_right {
                if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                    if !list.articles.is_empty() {
                        if app.viewer_article_idx == 0 {
                            app.viewer_article_idx = list.articles.len().saturating_sub(1);
                        } else {
                            app.viewer_article_idx -= 1;
                        }
                    }
                }
            } else if !app.saved_lists.lists.is_empty() {
                if app.viewer_list_idx == 0 {
                    app.viewer_list_idx = app.saved_lists.lists.len().saturating_sub(1);
                } else {
                    app.viewer_list_idx -= 1;
                }
                app.viewer_article_idx = 0;
            }
        }
        KeyCode::Enter => {
            if !app.viewer_focus_right {
                app.viewer_focus_right = true;
            } else {
                let target_article = app
                    .saved_lists
                    .lists
                    .get(app.viewer_list_idx)
                    .and_then(|list| list.articles.get(app.viewer_article_idx))
                    .cloned();

                if let Some(title) = target_article {
                    app.input_mode = InputMode::Normal;
                    app.open_article(&title);
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if app.viewer_focus_right {
                if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                    if list.id != "liked" {
                        if let Some(art) = list.articles.get(app.viewer_article_idx) {
                            app.pending_delete_is_list = false;
                            app.pending_delete_title = art.clone();
                            app.pending_delete_list_id = list.id.clone();
                            app.input_mode = InputMode::ConfirmDelete;
                        }
                    }
                }
            } else if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                if list.id != "liked" {
                    app.pending_delete_is_list = true;
                    app.pending_delete_title = list.name.clone();
                    app.pending_delete_list_id = list.id.clone();
                    app.input_mode = InputMode::ConfirmDelete;
                }
            }
        }
        KeyCode::Char('n') => {
            app.save_modal_target_title.clear();
            app.create_list_input.clear();
            app.create_list_return_mode = InputMode::SavedListsViewer;
            app.input_mode = InputMode::CreateNewList;
        }
        KeyCode::Char('M') | KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

pub fn handle_confirm_delete_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            if app.pending_delete_is_list {
                let list_id = app.pending_delete_list_id.clone();
                app.saved_lists.delete_list(&list_id);
                if app.viewer_list_idx > 0 {
                    app.viewer_list_idx -= 1;
                }
                app.viewer_article_idx = 0;
            } else {
                let list_id = app.pending_delete_list_id.clone();
                let title = app.pending_delete_title.clone();
                app.saved_lists.toggle_article_in_list(&list_id, &title);
                if app.viewer_article_idx > 0 {
                    app.viewer_article_idx -= 1;
                }
            }
            app.input_mode = InputMode::SavedListsViewer;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.input_mode = InputMode::SavedListsViewer;
        }
        _ => {}
    }
}
