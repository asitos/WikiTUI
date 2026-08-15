pub mod lists;
pub mod normal;
pub mod onboarding;
pub mod search;

use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_event(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
    match &app.input_mode {
        InputMode::CategoryOnboarding => onboarding::handle_category_onboarding_mode(app, key),
        InputMode::SaveToList => lists::handle_save_to_list_mode(app, key),
        InputMode::CreateNewList => lists::handle_create_new_list_mode(app, key),
        InputMode::SavedListsViewer => lists::handle_saved_lists_viewer_mode(app, key),
        InputMode::ConfirmDelete => lists::handle_confirm_delete_mode(app, key),
        InputMode::Help => match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::RestoreSessionPrompt(state) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let session = state.clone();
                app.restore_session(session);
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                crate::session::SessionState::delete_file();
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::LocalSearch => search::handle_local_search_mode(app, key, term_height),
        InputMode::Search => search::handle_search_mode(app, key),
        InputMode::Normal => normal::handle_normal_mode(app, key, term_width, term_height),
    }
}
