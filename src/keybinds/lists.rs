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
        KeyCode::Enter => {
            let name = app.create_list_input.trim().to_string();
            if !name.is_empty() {
                let list_id = app.saved_lists.create_list(&name);
                if !app.save_modal_target_title.is_empty() {
                    let target_title = app.save_modal_target_title.clone();
                    app.saved_lists
                        .toggle_article_in_list(&list_id, &target_title);
                }
            }
            app.input_mode = app.create_list_return_mode.clone();
        }
        KeyCode::Esc => {
            app.input_mode = app.create_list_return_mode.clone();
        }
        KeyCode::Backspace => {
            app.create_list_input.pop();
        }
        KeyCode::Char(c) => {
            app.create_list_input.push(c);
        }
        _ => {}
    }
}

pub fn handle_saved_lists_viewer_mode(app: &mut App, key: KeyEvent) {
    let lists_count = app.saved_lists.lists.len();
    let current_articles_count = app
        .saved_lists
        .lists
        .get(app.viewer_list_idx)
        .map(|l| l.articles.len())
        .unwrap_or(0);

    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            app.viewer_focus_right = false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if current_articles_count > 0 {
                app.viewer_focus_right = true;
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            if app.viewer_focus_right {
                if current_articles_count > 0 {
                    app.viewer_article_idx = (app.viewer_article_idx + 1) % current_articles_count;
                }
            } else if lists_count > 0 {
                app.viewer_list_idx = (app.viewer_list_idx + 1) % lists_count;
                app.viewer_article_idx = 0;
            }
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            if app.viewer_focus_right {
                if current_articles_count > 0 {
                    if app.viewer_article_idx == 0 {
                        app.viewer_article_idx = current_articles_count.saturating_sub(1);
                    } else {
                        app.viewer_article_idx -= 1;
                    }
                }
            } else if lists_count > 0 {
                if app.viewer_list_idx == 0 {
                    app.viewer_list_idx = lists_count.saturating_sub(1);
                } else {
                    app.viewer_list_idx -= 1;
                }
                app.viewer_article_idx = 0;
            }
        }
        KeyCode::Enter => {
            if app.viewer_focus_right {
                if let Some(title) = app
                    .saved_lists
                    .lists
                    .get(app.viewer_list_idx)
                    .and_then(|l| l.articles.get(app.viewer_article_idx))
                    .cloned()
                {
                    app.input_mode = InputMode::Normal;
                    app.open_article(&title);
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if app.viewer_focus_right {
                if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                    if !app.config.general.liked_readonly || list.id != "liked" {
                        if let Some(art) = list.articles.get(app.viewer_article_idx) {
                            app.confirm_action = Some(crate::app::ConfirmAction::DeleteArticle {
                                list_id: list.id.clone(),
                                title: art.clone(),
                            });
                            app.input_mode = InputMode::Confirm;
                        }
                    }
                }
            } else if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                if list.id != "liked" {
                    app.confirm_action = Some(crate::app::ConfirmAction::DeleteList {
                        list_id: list.id.clone(),
                        title: list.name.clone(),
                    });
                    app.input_mode = InputMode::Confirm;
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

pub fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            match app.confirm_action.take() {
                Some(crate::app::ConfirmAction::DeleteList { list_id, .. }) => {
                    app.saved_lists.delete_list(&list_id);
                    if app.viewer_list_idx > 0 {
                        app.viewer_list_idx -= 1;
                    }
                    app.viewer_article_idx = 0;
                    app.input_mode = InputMode::SavedListsViewer;
                }
                Some(crate::app::ConfirmAction::DeleteArticle { list_id, title }) => {
                    app.saved_lists.toggle_article_in_list(&list_id, &title);
                    if list_id == "liked" {
                        app.feed.profile.liked_articles.remove(&title);
                        if app.feed.profile.total_likes > 0 {
                            app.feed.profile.total_likes -= 1;
                        }
                        app.feed.profile.save();
                        for item in &mut app.feed.items {
                            if item.title == title {
                                item.is_liked = false;
                            }
                        }
                    }
                    if app.viewer_article_idx > 0 {
                        app.viewer_article_idx -= 1;
                    }
                    app.input_mode = InputMode::SavedListsViewer;
                }
                Some(crate::app::ConfirmAction::ResetFeed) => {
                    app.reset_feed();
                }
                None => {
                    app.input_mode = InputMode::Normal;
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            let action = app.confirm_action.take();
            if matches!(action, Some(crate::app::ConfirmAction::ResetFeed)) {
                app.input_mode = InputMode::Normal;
            } else {
                app.input_mode = InputMode::SavedListsViewer;
            }
        }
        _ => {}
    }
}
