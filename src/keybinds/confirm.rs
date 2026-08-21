use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => match app.confirm_action.take() {
            Some(crate::app::ConfirmAction::DeleteList { list_id, .. }) => {
                app.saved_lists.delete_list(&list_id);
                if app.lists_modal.viewer_list_idx > 0 {
                    app.lists_modal.viewer_list_idx -= 1;
                }
                app.lists_modal.viewer_article_idx = 0;
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
                if app.lists_modal.viewer_article_idx > 0 {
                    app.lists_modal.viewer_article_idx -= 1;
                }
                app.input_mode = InputMode::SavedListsViewer;
            }
            Some(crate::app::ConfirmAction::ResetFeed) => {
                app.reset_feed();
            }
            Some(crate::app::ConfirmAction::Quit) => {
                app.save_session();
                app.running = false;
            }
            None => {
                app.input_mode = InputMode::Normal;
            }
        },
        KeyCode::Char('n') | KeyCode::Esc => {
            let action = app.confirm_action.take();
            if matches!(
                action,
                Some(crate::app::ConfirmAction::ResetFeed) | Some(crate::app::ConfirmAction::Quit)
            ) {
                app.input_mode = InputMode::Normal;
            } else {
                app.input_mode = InputMode::SavedListsViewer;
            }
        }
        _ => {}
    }
}
