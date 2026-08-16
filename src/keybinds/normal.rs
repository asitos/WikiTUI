use crate::app::App;
use crate::layout::SplitDirection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_normal_mode(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
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
                app.toggle_feed_like();
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
            KeyCode::Char('c') | KeyCode::Char('x') | KeyCode::Char('q') => {
                app.close_active_pane();
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                app.clear_local_search();
            }
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
            KeyCode::Char('y') => {
                app.copy_focused_link();
            }
            KeyCode::Char('Y') => {
                app.copy_article_link();
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
            KeyCode::Char('t')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.new_tab();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.waiting_for_split_cmd = true;
            }
            KeyCode::Char('S') => {
                if app.active_tab().name == "home" {
                    if let Some(session) = crate::session::SessionState::load() {
                        session.restore_to_app(app);
                    }
                }
            }
            KeyCode::Char('H') | KeyCode::Backspace => {
                app.history_back(term_height);
            }
            KeyCode::Char('L') => {
                app.history_forward();
            }
            KeyCode::Char('h')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.prev_tab();
            }
            KeyCode::Char('l')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.next_tab();
            }
            KeyCode::Char(c @ '0'..='9')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                let tab_idx = if c == '0' {
                    9
                } else {
                    (c as usize) - ('1' as usize)
                };
                app.switch_to_tab(tab_idx);
            }
            KeyCode::Char('x') => {
                app.close_active_pane();
            }
            KeyCode::Char('C')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.reopen_last_closed();
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
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
                app.activate_selected(term_height);
            }
            _ => {}
        }
    }
}
