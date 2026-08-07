use crate::app::{App, InputMode};
use crate::layout::SplitDirection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
    match app.input_mode {
        InputMode::Help => match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::LocalSearch => match key.code {
            KeyCode::Char(c) => {
                let pane = app.active_pane_mut();
                pane.local_search_query.push(c);
                app.update_local_search();
            }
            KeyCode::Backspace => {
                let pane = app.active_pane_mut();
                pane.local_search_query.pop();
                app.update_local_search();
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::Search => match key.code {
            KeyCode::Char(c) => {
                app.type_search_char(c);
            }
            KeyCode::Backspace => {
                app.backspace_search_char();
            }
            KeyCode::Enter => {
                app.submit_search();
            }
            KeyCode::Esc => {
                app.exit_search_mode();
            }
            _ => {}
        },
        InputMode::Normal => {
            if app.waiting_for_split_cmd {
                app.waiting_for_split_cmd = false;
                match key.code {
                    KeyCode::Char('v') => {
                        app.split_active_pane(SplitDirection::Vertical);
                    }
                    KeyCode::Char('s') => {
                        app.split_active_pane(SplitDirection::Horizontal);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => {
                        app.quit();
                    }
                    KeyCode::Char('?') => {
                        app.toggle_help_popup();
                    }
                    KeyCode::Char('/') => {
                        app.enter_local_search_mode();
                    }
                    KeyCode::Char('n') => {
                        app.next_local_match();
                    }
                    KeyCode::Char('N') => {
                        app.prev_local_match();
                    }
                    KeyCode::Char(']') => {
                        app.jump_next_heading(term_height);
                    }
                    KeyCode::Char('[') => {
                        app.jump_prev_heading(term_height);
                    }
                    KeyCode::Char('f') => {
                        app.scroll_page_down(term_height);
                    }
                    KeyCode::Char('b') => {
                        app.scroll_page_up(term_height);
                    }
                    KeyCode::Char('g') => {
                        app.jump_to_top();
                    }
                    KeyCode::Char('G') => {
                        app.jump_to_bottom(term_height);
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.enter_search_mode();
                    }
                    KeyCode::Char('i') => {
                        app.edit_search_mode();
                    }
                    KeyCode::Char('s') => {
                        app.activate_selected_in_split(SplitDirection::Horizontal);
                    }
                    KeyCode::Char('v') => {
                        app.activate_selected_in_split(SplitDirection::Vertical);
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.new_tab();
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.waiting_for_split_cmd = true;
                    }
                    KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.prev_tab();
                    }
                    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.next_tab();
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.close_active_pane();
                    }
                    KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.navigate_panes('h', term_width, term_height);
                    }
                    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.navigate_panes('l', term_width, term_height);
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.navigate_panes('j', term_width, term_height);
                    }
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.navigate_panes('k', term_width, term_height);
                    }
                    KeyCode::Tab => {
                        app.focus_next_link();
                    }
                    KeyCode::BackTab => {
                        app.focus_prev_link();
                    }
                    KeyCode::Char('j') => {
                        app.select_next_item(term_height);
                    }
                    KeyCode::Char('k') => {
                        app.select_prev_item(term_height);
                    }
                    KeyCode::Char('t') => {
                        app.activate_selected_in_new_tab();
                    }
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.activate_selected_in_new_tab();
                    }
                    KeyCode::Enter => {
                        app.activate_selected();
                    }
                    _ => {}
                }
            }
        }
    }
}
