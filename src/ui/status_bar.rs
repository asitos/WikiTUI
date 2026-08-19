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
            if total_lines == 0 || scroll == 0 {
                "TOP".to_string()
            } else if scroll + (area.height as usize) >= total_lines {
                "BOT".to_string()
            } else {
                format!("{}%", (scroll * 100) / total_lines.max(1))
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

    // center segment
    let (center_text, center_style) = if let Some((ref msg, time)) = app.status_message {
        if time.elapsed().as_secs_f32() < 3.0 {
            (
                format!("✓ {}", msg),
                Style::default()
                    .fg(theme::LIME)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            get_center_hints(app, active_pane)
        }
    } else {
        get_center_hints(app, active_pane)
    };

    let center_spans = vec![Span::styled(center_text, center_style)];

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

fn get_center_hints(app: &App, active_pane: &crate::app::Pane) -> (String, Style) {
    match app.input_mode {
        InputMode::Search => (
            "type query · enter search · esc cancel".to_string(),
            Style::default()
                .fg(theme::BEIGE)
                .add_modifier(Modifier::BOLD),
        ),
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
            (
                format!(
                    "/ {}_ · {} · n next · N prev · esc exit",
                    active_pane.local_search_query, matches_info
                ),
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )
        }
        InputMode::CategoryOnboarding => (
            "j/k navigate · space toggle · enter start feed".to_string(),
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Help => ("".to_string(), Style::default().fg(theme::GREY)),
        InputMode::SaveToList => (
            "j/k navigate · space toggle · c new list · esc done".to_string(),
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::CreateNewList => (
            "enter confirm · esc cancel".to_string(),
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::SavedListsViewer => (
            "h/l switch pane · j/k navigate · enter open · d delete".to_string(),
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Confirm => ("".to_string(), Style::default().fg(theme::RED)),
        InputMode::Settings => (
            "j/k navigate · space/enter toggle · h/l adjust · r reset · esc close".to_string(),
            Style::default()
                .fg(theme::ORANGE)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Normal => {
            if app.feed.active {
                (
                    "j/k browse · l like · enter read · t tab · r reset · esc exit"
                        .to_string(),
                    Style::default().fg(theme::VIOLET),
                )
            } else if active_pane.toc_focused {
                (
                    "j/k navigate contents · enter jump · o close".to_string(),
                    Style::default()
                        .fg(theme::LIME)
                        .add_modifier(Modifier::BOLD),
                )
            } else if let Some(link) = active_pane.focused_link().filter(|l| l.is_external()) {
                (
                    format!("↗ external {} · enter/y copy URL", link.title),
                    Style::default()
                        .fg(theme::TEAL)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "ctrl-s search · r random · F feed · , settings · ? help · q quit"
                        .to_string(),
                    Style::default().fg(theme::GREY),
                )
            }
        }
    }
}
