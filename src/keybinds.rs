use crate::app::{App, InputMode};
use crate::layout::SplitDirection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
    match app.input_mode {
        InputMode::CategoryOnboarding => match key.code {
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                let total = crate::feed::profile::POPULAR_CATEGORIES.len();
                app.onboarding_cursor_idx = (app.onboarding_cursor_idx + 1) % total;
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                let total = crate::feed::profile::POPULAR_CATEGORIES.len();
                if app.onboarding_cursor_idx == 0 {
                    app.onboarding_cursor_idx = total.saturating_sub(1);
                } else {
                    app.onboarding_cursor_idx -= 1;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(val) = app.onboarding_selected.get_mut(app.onboarding_cursor_idx) {
                    *val = !*val;
                }
            }
            KeyCode::Enter => {
                app.submit_category_onboarding();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.toggle_feed_mode();
            }
            _ => {}
        },
        InputMode::SaveToList => match key.code {
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                let total = app.saved_lists.lists.len() + 1;
                app.save_modal_cursor_idx = (app.save_modal_cursor_idx + 1) % total;
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                let total = app.saved_lists.lists.len() + 1;
                if app.save_modal_cursor_idx == 0 {
                    app.save_modal_cursor_idx = total.saturating_sub(1);
                } else {
                    app.save_modal_cursor_idx -= 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                let list_count = app.saved_lists.lists.len();
                if app.save_modal_cursor_idx < list_count {
                    let list_id = app.saved_lists.lists[app.save_modal_cursor_idx].id.clone();
                    let target_title = app.save_modal_target_title.clone();
                    let target_snippet = app.save_modal_target_snippet.clone();
                    app.saved_lists.toggle_article_in_list(
                        &list_id,
                        &target_title,
                        target_snippet.as_deref(),
                    );
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
        },
        InputMode::CreateNewList => match key.code {
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
        },
        InputMode::SavedListsViewer => match key.code {
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
                            app.viewer_article_idx =
                                (app.viewer_article_idx + 1) % list.articles.len();
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
                        .map(|art| art.title.clone());

                    if let Some(title) = target_article {
                        app.input_mode = InputMode::Normal;
                        app.open_article(&title);
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if app.viewer_focus_right {
                    if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                        if let Some(art) = list.articles.get(app.viewer_article_idx) {
                            app.pending_delete_is_list = false;
                            app.pending_delete_title = art.title.clone();
                            app.pending_delete_list_id = list.id.clone();
                            app.input_mode = InputMode::ConfirmDelete;
                        }
                    }
                } else if let Some(list) = app.saved_lists.lists.get(app.viewer_list_idx) {
                    app.pending_delete_is_list = true;
                    app.pending_delete_title = list.name.clone();
                    app.pending_delete_list_id = list.id.clone();
                    app.input_mode = InputMode::ConfirmDelete;
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
        },
        InputMode::ConfirmDelete => match key.code {
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
                    app.saved_lists.toggle_article_in_list(&list_id, &title, None);
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
        },
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
                app.update_local_search(term_height);
            }
            KeyCode::Backspace => {
                let pane = app.active_pane_mut();
                pane.local_search_query.pop();
                app.update_local_search(term_height);
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::Search => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('w') | KeyCode::Char('h') | KeyCode::Backspace => {
                        app.delete_word_left();
                        return;
                    }
                    _ => {}
                }
            }
            match key.code {
                KeyCode::Char(c) => {
                    app.type_search_char(c);
                }
                KeyCode::Backspace => {
                    app.backspace_search_char();
                }
                KeyCode::Delete => {
                    app.delete_search_char();
                }
                KeyCode::Left => {
                    app.move_search_cursor_left();
                }
                KeyCode::Right => {
                    app.move_search_cursor_right();
                }
                KeyCode::Home => {
                    app.move_search_cursor_home();
                }
                KeyCode::End => {
                    app.move_search_cursor_end();
                }
                KeyCode::Enter => {
                    app.submit_search();
                }
                KeyCode::Esc => {
                    app.exit_search_mode();
                }
                _ => {}
            }
        },
        InputMode::Normal => {
            if app.feed.active {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('F') | KeyCode::Char('q') => {
                        app.feed.active = false;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.feed.next_post();
                        app.maybe_fetch_feed_batch();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.feed.prev_post();
                    }
                    KeyCode::Char('l') => {
                        app.feed.toggle_like();
                    }
                    KeyCode::Enter => {
                        if let Some(item) = app.feed.current_item().cloned() {
                            app.feed.active = false;
                            let pane_id = app.active_pane().id;
                            app.active_pane_mut().is_loading = true;
                            let _ = app.cmd_tx.send(crate::api::NetworkCommand::FetchArticle {
                                pane_id,
                                title: item.title,
                            });
                        }
                    }
                    KeyCode::Char('t') => {
                        if let Some(item) = app.feed.current_item().cloned() {
                            app.feed.active = false;
                            app.new_tab();
                            let pane_id = app.active_pane().id;
                            app.active_pane_mut().is_loading = true;
                            let _ = app.cmd_tx.send(crate::api::NetworkCommand::FetchArticle {
                                pane_id,
                                title: item.title,
                            });
                        }
                    }
                    _ => {}
                }
            } else if app.active_pane().toc_focused {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('o') => {
                        app.toggle_toc();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.select_next_toc_item();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.select_prev_toc_item();
                    }
                    KeyCode::Enter => {
                        app.activate_toc_selection(term_height);
                    }
                    _ => {}
                }
            } else if app.waiting_for_split_cmd {
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
                    KeyCode::Char('z') => {
                        app.toggle_zen_mode();
                    }
                    KeyCode::Char('F') => {
                        app.toggle_feed_mode();
                    }
                    KeyCode::Char('r') => {
                        app.fetch_random_article();
                    }
                    KeyCode::Char('o') => {
                        app.toggle_toc();
                    }
                    KeyCode::Char('m') => {
                        app.open_save_to_list_modal();
                    }
                    KeyCode::Char('M') => {
                        app.open_saved_lists_viewer();
                    }
                    KeyCode::Char('?') => {
                        app.toggle_help_popup();
                    }
                    KeyCode::Char('/') => {
                        app.enter_local_search_mode();
                    }
                    KeyCode::Char('n') => {
                        app.next_local_match(term_height);
                    }
                    KeyCode::Char('N') => {
                        app.prev_local_match(term_height);
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
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.select_next_item(term_height);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
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
