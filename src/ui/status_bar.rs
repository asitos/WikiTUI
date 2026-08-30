use crate::app::{App, InputMode, PaneContent};
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if area.width < 10 || area.height == 0 {
        return;
    }

    let active_pane = app.active_pane();

    // left segment
    let (badge_text, badge_color) = if app.feed.active {
        (" FEED ", theme::PINK)
    } else {
        match app.input_mode {
            InputMode::Normal => (" NORMAL ", theme::LIME),
            InputMode::Search => (" SEARCH ", theme::BEIGE),
            InputMode::LocalSearch => (" FIND ", theme::YELLOW),
            InputMode::CategoryOnboarding => (" SETUP ", theme::VIOLET),
            InputMode::SaveToList | InputMode::SavedListsViewer | InputMode::CreateNewList => {
                (" LISTS ", theme::VIOLET)
            }
            InputMode::Settings => (" CONFIG ", theme::ORANGE),
            InputMode::Help => (" HELP ", theme::GREY),
            InputMode::Confirm => (" PROMPT ", theme::RED),
            InputMode::Categories => (" CATEGORIES ", theme::TEAL),
            InputMode::DailyFeedModal => (" FEED ", theme::TEAL),
        }
    };

    let left_spans = vec![Span::styled(
        badge_text,
        Style::default()
            .fg(theme::BG)
            .bg(badge_color)
            .add_modifier(Modifier::BOLD),
    )];
    let left_width = badge_text.chars().count() as u16 + 2;

    // right segment
    let right_text = match &active_pane.content {
        PaneContent::ArticleText { parsed_doc, .. } => {
            let total_lines = parsed_doc.lines.len();
            let scroll = active_pane.scroll_offset;
            let viewport = active_pane.viewport_height.max(1);
            let max_scroll = total_lines.saturating_sub(viewport);

            if total_lines <= viewport {
                "ALL".to_string()
            } else if scroll == 0 {
                "TOP".to_string()
            } else if scroll >= max_scroll {
                "BOT".to_string()
            } else {
                format!("{}%", (scroll * 100) / max_scroll.max(1))
            }
        }
        PaneContent::SearchResults { items, .. } => {
            if !items.is_empty() {
                format!("{}/{}", active_pane.selected_idx + 1, items.len())
            } else {
                String::new()
            }
        }
        PaneContent::Empty => {
            format!("v{}", env!("CARGO_PKG_VERSION"))
        }
        PaneContent::Error(_) => "ERR".to_string(),
    };

    let right_spans = vec![Span::styled(
        format!(" {} ", right_text),
        Style::default()
            .fg(theme::GREY)
            .add_modifier(Modifier::ITALIC),
    )];
    let right_width = right_text.chars().count() as u16 + 3;

    let center_width = (area.width as usize).saturating_sub((left_width + right_width) as usize);

    // center segment
    let center_spans = if let Some((ref msg, time)) = app.status_message {
        if time.elapsed().as_secs_f32() < 3.0 {
            vec![Span::styled(
                msg.clone(),
                Style::default()
                    .fg(theme::LIME)
                    .add_modifier(Modifier::BOLD),
            )]
        } else {
            get_center_spans(app, active_pane, center_width)
        }
    } else {
        get_center_spans(app, active_pane, center_width)
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_width),
            Constraint::Min(0),
            Constraint::Length(right_width),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(center_spans)).alignment(Alignment::Center),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[2],
    );
}

fn get_center_spans(
    app: &App,
    active_pane: &crate::app::Pane,
    available_width: usize,
) -> Vec<Span<'static>> {
    match app.input_mode {
        InputMode::Search => vec![Span::styled(
            "type query · enter search · esc cancel",
            Style::default()
                .fg(theme::BEIGE)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::LocalSearch => {
            let matches_info = if active_pane.local_matches.is_empty() {
                "no matches".to_string()
            } else {
                format!(
                    "match {}/{}",
                    active_pane.selected_match_idx.unwrap_or(0) + 1,
                    active_pane.local_matches.len()
                )
            };
            vec![Span::styled(
                format!(
                    "/: {}_ · {} · n next · N prev · esc exit",
                    active_pane.local_search_query, matches_info
                ),
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )]
        }
        InputMode::CategoryOnboarding => vec![Span::styled(
            "j/k navigate · space toggle · enter start feed",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Help => vec![Span::styled(
            "esc/q/? close",
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::SaveToList => vec![Span::styled(
            "j/k navigate · space toggle · c new list · esc done",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::CreateNewList => vec![Span::styled(
            "enter confirm · esc cancel",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::SavedListsViewer => vec![Span::styled(
            "h/l switch pane · j/k navigate · enter open · d delete · esc close",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Confirm => vec![Span::styled(
            "y/enter confirm · n/esc cancel",
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
        )],
        InputMode::Settings => vec![Span::styled(
            "j/k navigate · space/enter toggle · h/l adjust · r reset · esc close",
            Style::default()
                .fg(theme::ORANGE)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Categories => vec![Span::styled(
            "j/k navigate · enter open category · y copy URL · esc close",
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::DailyFeedModal => {
            if let Some(modal) = &app.daily_feed_modal {
                if modal.kind == crate::ui::modals::DailyFeedKind::OnThisDay {
                    vec![Span::styled(
                        "1-4 category · j/k navigate · tab links · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                } else if modal.kind == crate::ui::modals::DailyFeedKind::News {
                    vec![Span::styled(
                        "j/k navigate · tab links · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                } else {
                    vec![Span::styled(
                        "j/k navigate · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                }
            } else {
                vec![Span::styled(
                    "j/k navigate · enter read · esc close",
                    Style::default()
                        .fg(theme::BLUE)
                        .add_modifier(Modifier::BOLD),
                )]
            }
        }
        InputMode::Normal => {
            if app.audio_player.is_active() {
                let state_str = match app.audio_player.state {
                    crate::audio::PlaybackState::Playing => "󰎆 playing",
                    crate::audio::PlaybackState::Paused => "󰏤 paused",
                    _ => "audio",
                };
                let title = app
                    .audio_player
                    .current_title
                    .as_deref()
                    .unwrap_or("article");
                vec![
                    Span::styled(
                        format!("{} [{}]", state_str, title),
                        Style::default()
                            .fg(theme::PINK)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " · a pause/resume · A stop",
                        Style::default().fg(theme::GREY),
                    ),
                ]
            } else if app.feed.active {
                vec![Span::styled(
                    "j/k browse · l like · enter read · t tab · r reset · esc exit",
                    Style::default().fg(theme::GREY),
                )]
            } else if active_pane.show_toc || active_pane.toc_focused {
                vec![Span::styled(
                    "j/k navigate contents · enter jump · esc/o close",
                    Style::default()
                        .fg(theme::LIME)
                        .add_modifier(Modifier::BOLD),
                )]
            } else if let Some(link) = active_pane.focused_link().filter(|l| l.is_external()) {
                vec![Span::styled(
                    format!("↗ external {} · enter/y copy URL", link.title),
                    Style::default()
                        .fg(theme::TEAL)
                        .add_modifier(Modifier::BOLD),
                )]
            } else if matches!(active_pane.content, PaneContent::ArticleText { .. })
                && (!active_pane.history_back.is_empty() || !active_pane.history_forward.is_empty())
            {
                build_history_trail(active_pane, available_width)
            } else if matches!(active_pane.content, PaneContent::Empty) {
                vec![Span::styled(
                    "ctrl-s search · F feed · , settings · ? help · q quit",
                    Style::default().fg(theme::GREY),
                )]
            } else {
                let has_spoken =
                    if let PaneContent::ArticleText { parsed_doc, .. } = &active_pane.content {
                        parsed_doc.spoken_audio.is_some()
                    } else {
                        false
                    };
                if has_spoken {
                    vec![Span::styled(
                        "ctrl-s search · a listen · r random · F feed · , settings · ? help",
                        Style::default().fg(theme::GREY),
                    )]
                } else {
                    vec![Span::styled(
                        "ctrl-s search · r random · F feed · , settings · ? help · q quit",
                        Style::default().fg(theme::GREY),
                    )]
                }
            }
        }
    }
}

fn build_history_trail(pane: &crate::app::Pane, available_width: usize) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let back_len = pane.history_back.len();
    let start_idx = back_len.saturating_sub(2);
    let back_items = &pane.history_back[start_idx..];
    let fwd_items: Vec<_> = pane.history_forward.iter().take(2).collect();

    let total_items = 1 + back_items.len() + fwd_items.len();
    let overhead = total_items * 3
        + if start_idx > 0 { 4 } else { 0 }
        + if pane.history_forward.len() > 2 { 4 } else { 0 };
    let budget = available_width.saturating_sub(overhead);
    let side_max = (budget / total_items.max(1)).clamp(6, 35);
    let cur_max = (side_max + side_max / 3).clamp(8, 45);

    if start_idx > 0 {
        spans.push(Span::styled("…", Style::default().fg(theme::GREY)));
        spans.push(Span::styled(" › ", Style::default().fg(theme::GREY)));
    }

    for title in back_items {
        spans.push(Span::styled(
            truncate_trail_title(title, side_max),
            Style::default().fg(theme::GREY),
        ));
        spans.push(Span::styled(" › ", Style::default().fg(theme::GREY)));
    }

    if let Some(cur_title) = pane.title() {
        spans.push(Span::styled(
            truncate_trail_title(&cur_title, cur_max),
            Style::default()
                .fg(theme::LIME)
                .add_modifier(Modifier::BOLD),
        ));
    }

    for title in fwd_items {
        spans.push(Span::styled(" › ", Style::default().fg(theme::GREY)));
        spans.push(Span::styled(
            truncate_trail_title(title, side_max),
            Style::default()
                .fg(theme::GREY)
                .add_modifier(Modifier::ITALIC),
        ));
    }

    if pane.history_forward.len() > 2 {
        spans.push(Span::styled(" › …", Style::default().fg(theme::GREY)));
    }

    spans
}

fn truncate_trail_title(title: &str, max_len: usize) -> String {
    let lower = title.to_lowercase();
    crate::ui::truncate_with_ellipsis(&lower, max_len, "…")
}
