use crate::app::{App, InputMode};
use crate::ui::modals::{get_feed_entries, parse_story_html, DailyFeedKind};
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
                        modal.link_idx = 0;
                    }
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(modal) = &mut app.daily_feed_modal {
                if modal.cursor_idx > 0 {
                    modal.cursor_idx -= 1;
                    modal.link_idx = 0;
                }
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            if let Some(modal) = &mut app.daily_feed_modal {
                modal.cursor_idx = 0;
                modal.link_idx = 0;
            }
        }
        KeyCode::Char('G') | KeyCode::End => {
            if total > 0 {
                if let Some(modal) = &mut app.daily_feed_modal {
                    modal.cursor_idx = total.saturating_sub(1);
                    modal.link_idx = 0;
                }
            }
        }
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
            if state.kind == DailyFeedKind::News {
                if let Some(feed) = &app.daily_feed {
                    if let Some(item) = feed.news.get(state.cursor_idx) {
                        let raw = item.story.as_deref().unwrap_or("");
                        let (_, links) = parse_story_html(raw);
                        let total_links = links.len();
                        if total_links > 0 {
                            if let Some(modal) = &mut app.daily_feed_modal {
                                modal.link_idx = (modal.link_idx + 1) % total_links;
                            }
                        }
                    }
                }
            }
        }
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
            if state.kind == DailyFeedKind::News {
                if let Some(feed) = &app.daily_feed {
                    if let Some(item) = feed.news.get(state.cursor_idx) {
                        let raw = item.story.as_deref().unwrap_or("");
                        let (_, links) = parse_story_html(raw);
                        let total_links = links.len();
                        if total_links > 0 {
                            if let Some(modal) = &mut app.daily_feed_modal {
                                modal.link_idx = if modal.link_idx == 0 {
                                    total_links - 1
                                } else {
                                    modal.link_idx - 1
                                };
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Enter => {
            let target = if state.kind == DailyFeedKind::News {
                if let Some(feed) = &app.daily_feed {
                    feed.news.get(state.cursor_idx).and_then(|item| {
                        let raw = item.story.as_deref().unwrap_or("");
                        let (_, links) = parse_story_html(raw);
                        links
                            .get(state.link_idx)
                            .cloned()
                            .or_else(|| links.first().cloned())
                    })
                } else {
                    None
                }
            } else {
                entries
                    .get(state.cursor_idx)
                    .map(|e| e.target_article.clone())
            };

            if let Some(target) =
                target.or_else(|| entries.get(state.cursor_idx).map(|e| e.target_article.clone()))
            {
                if !target.is_empty() {
                    app.close_daily_feed_modal();
                    app.open_article(&target);
                }
            }
        }
        KeyCode::Char('t') => {
            let target = if state.kind == DailyFeedKind::News {
                if let Some(feed) = &app.daily_feed {
                    feed.news.get(state.cursor_idx).and_then(|item| {
                        let raw = item.story.as_deref().unwrap_or("");
                        let (_, links) = parse_story_html(raw);
                        links
                            .get(state.link_idx)
                            .cloned()
                            .or_else(|| links.first().cloned())
                    })
                } else {
                    None
                }
            } else {
                entries
                    .get(state.cursor_idx)
                    .map(|e| e.target_article.clone())
            };

            if let Some(target) =
                target.or_else(|| entries.get(state.cursor_idx).map(|e| e.target_article.clone()))
            {
                if !target.is_empty() {
                    app.close_daily_feed_modal();
                    if !matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                        app.new_tab();
                    }
                    app.open_article(&target);
                }
            }
        }
        _ => {}
    }
}