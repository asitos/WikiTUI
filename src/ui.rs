use crate::app::{App, InputMode, PaneContent};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Line, Span, Style, Stylize, Modifier},
    widgets::{Block, Clear, Paragraph},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top tab bar
            Constraint::Min(0),    // main workspace
            Constraint::Length(1), // bottom status bar
        ])
        .split(size);

    let tab_bar_area = chunks[0];
    let main_area = chunks[1];
    let status_area = chunks[2];

    // render tabs
    let mut tab_spans = Vec::new();
    tab_spans.push(Span::styled(" [ ", Style::default().fg(theme::GREY)));
    for (i, tab) in app.tabs.iter().enumerate() {
        let is_active = i == app.active_tab_idx;
        let style = if is_active {
            Style::default().fg(theme::LIME).bold()
        } else {
            Style::default().fg(theme::GREY)
        };

        let tab_title = if let Some(active_pane) = tab.panes.get(tab.active_pane_idx) {
            match &active_pane.content {
                PaneContent::ArticleText { title, .. } => title.to_lowercase(),
                PaneContent::SearchResults { query, .. } => {
                    format!("search: {}", query.to_lowercase())
                }
                PaneContent::Error(_) => "error".to_string(),
                PaneContent::Empty => tab.name.to_lowercase(),
            }
        } else {
            tab.name.to_lowercase()
        };

        tab_spans.push(Span::styled(
            tab_title,
            style,
        ));
        if i < app.tabs.len() - 1 {
            tab_spans.push(Span::styled(" | ", Style::default().fg(theme::GREY)));
        }
    }
    tab_spans.push(Span::styled(" ] ", Style::default().fg(theme::GREY)));
    let tab_bar_paragraph = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, tab_bar_area);

    // status bar
    let status_text = match app.input_mode {
        // for when in search mode
        InputMode::Search => " search wikipedia ".to_string(),
        // for when in local search mode
        InputMode::LocalSearch => {
            let active_pane = app.active_pane();
            let matches_info = if active_pane.local_matches.is_empty() {
                "no matches".to_string()
            } else {
                format!(
                    "match {}/{}",
                    active_pane.selected_match_idx.unwrap_or(0) + 1,
                    active_pane.local_matches.len()
                )
            };
            format!(
                " /: {}_ | {} | n: next | N: prev | esc: exit ",
                active_pane.local_search_query, matches_info
            )
        }
        InputMode::Help => {
            "".to_string()
        }
        // normally
        InputMode::Normal => {
            "ctrl-s: search | ?: help | q: quit".to_string()
        }
    };
    let status_style = match app.input_mode {
        InputMode::Search => Style::default().fg(theme::BEIGE).bold(),
        InputMode::LocalSearch => Style::default().fg(theme::YELLOW).bold(),
        InputMode::Help => Style::default().fg(theme::GREY).bold(),
        InputMode::Normal => Style::default().fg(theme::GREY),
    };
    let status_paragraph = Paragraph::new(status_text)
        .style(status_style)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(status_paragraph, status_area);

    // panes
    let active_tab_idx = app.active_tab_idx;
    let rects = app.tabs[active_tab_idx]
        .layout_root
        .compute_rects(main_area);
    let active_pane_idx = app.tabs[active_tab_idx].active_pane_idx;

    for (pane_idx, rect) in rects {
        let is_active = pane_idx == active_pane_idx;
        let content_width = rect.width.saturating_sub(2) as usize;

        let pane = &mut app.tabs[active_tab_idx].panes[pane_idx];
        pane.ensure_parsed_width(content_width);
        pane.viewport_height = rect.height.saturating_sub(2) as usize;

        let border_color = if is_active {
            theme::PINK
        } else {
            theme::DARK_GREY
        };

        // determine title for pane
        let title = match &pane.content {
            PaneContent::Empty => String::new(),
            PaneContent::SearchResults { query, .. } => {
                format!(" search: {} ", query.to_lowercase())
            }
            PaneContent::ArticleText { title, .. } => {
                format!(" {} ", title.to_lowercase())
            }
            PaneContent::Error(_) => " error ".to_string(),
        };

        let block = Block::bordered()
            .border_style(Style::default().fg(border_color))
            .title(title);

        if pane.is_loading {
            let loading_p = Paragraph::new(" loading wikipedia data... ")
                .fg(theme::YELLOW)
                .block(block);
            f.render_widget(loading_p, rect);
            continue;
        }

        match &pane.content {
            PaneContent::Empty => {
                let empty_p = Paragraph::new("")
                    .fg(theme::GREY)
                    .block(block)
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(empty_p, rect);
            }
            PaneContent::SearchResults { items, .. } => {
                if items.is_empty() {
                    let no_res_p = Paragraph::new("no search results found")
                        .fg(theme::RED)
                        .block(block);
                    f.render_widget(no_res_p, rect);
                } else {
                    let mut lines = Vec::new();
                    for (i, item) in items.iter().enumerate() {
                        let is_selected = i == pane.selected_idx;
                        let prefix = if is_selected { "> " } else { "  " };
                        let title_style = if is_selected {
                            Style::default().fg(theme::LIME).bold()
                        } else {
                            Style::default().fg(theme::FG).bold()
                        };

                        lines.push(Line::from(vec![
                            Span::styled(prefix, title_style),
                            Span::styled(format!("{}. {}", i + 1, item.title), title_style),
                        ]));

                        if !item.snippet.is_empty() {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(&item.snippet, Style::default().fg(theme::GREY)),
                            ]));
                        }
                        lines.push(Line::from(""));
                    }
                    let results_p = Paragraph::new(lines)
                        .block(block)
                        .scroll((pane.scroll_offset as u16, 0));
                    f.render_widget(results_p, rect);
                }
            }
            PaneContent::ArticleText { parsed_doc, .. } => {
                let mut rendered_lines = parsed_doc.lines.clone();

                if let Some((link, line)) = pane
                    .selected_link_idx
                    .and_then(|idx| parsed_doc.links.get(idx))
                    .and_then(|link| {
                        rendered_lines
                            .get_mut(link.line_idx)
                            .map(|line| (link, line))
                    })
                {
                    for &span_idx in &link.span_indices {
                        if let Some(span) = line.spans.get_mut(span_idx) {
                            span.style = Style::default()
                                // focused link
                                .fg(theme::BLUE)
                                .bold()
                                .add_modifier(Modifier::UNDERLINED);
                        }
                    }
                }

                // highlight local search matches
                if !pane.local_search_query.trim().is_empty() {
                    let active_match = pane
                        .selected_match_idx
                        .and_then(|idx| pane.local_matches.get(idx));

                    for (m_idx, m) in pane.local_matches.iter().enumerate() {
                        let span = rendered_lines
                            .get_mut(m.line_idx)
                            .and_then(|line| line.spans.get_mut(m.span_idx));

                        if let Some(span) = span {
                            let is_active_match = active_match.is_some_and(|active| {
                                active.line_idx == m.line_idx
                                    && active.span_idx == m.span_idx
                                    && pane.selected_match_idx == Some(m_idx)
                            });

                            if is_active_match {
                                span.style =
                                    Style::default().bg(theme::YELLOW).fg(theme::BG).bold();
                            } else {
                                span.style = Style::default().bg(theme::BEIGE).fg(theme::BG);
                            }
                        }
                    }
                }

                let paragraph = Paragraph::new(rendered_lines)
                    .block(block)
                    .scroll((pane.scroll_offset as u16, 0));
                f.render_widget(paragraph, rect);
            }
            PaneContent::Error(err_msg) => {
                let err_p = Paragraph::new(format!("error: {}", err_msg))
                    .fg(theme::RED)
                    .block(block);
                f.render_widget(err_p, rect);
            }
        }
    }

    // help popup
    if app.input_mode == InputMode::Help {
        let area = centered_rect(70, 85, size);
        f.render_widget(Clear, area);

        let help_text = vec![
            Line::from(vec![Span::styled("navigation", Style::default().fg(theme::VIOLET).bold())]),
            Line::from("  j/k            scroll down / up"),
            Line::from("  f/b            scroll page down / up"),
            Line::from("  g/G            jump to top / bottom"),
            Line::from("  ]/[            jump to next / prev section heading"),
            Line::from(""),
            Line::from(vec![Span::styled("links & selection", Style::default().fg(theme::VIOLET).bold())]),
            Line::from("  tab/backtab    focus next / prev link"),
            Line::from("  enter          open link in current pane"),
            Line::from("  t              open link in new tab"),
            Line::from("  s/v            open link in horizontal / vertical split"),
            Line::from(""),
            Line::from(vec![Span::styled("panes & tabs", Style::default().fg(theme::VIOLET).bold())]),
            Line::from("  ctrl-w s/v     split active pane horizontally / vertically"),
            Line::from("  ctrl-h/j/k/l   navigate focus between split panes"),
            Line::from("  alt-c          close active pane"),
            Line::from("  ctrl-t         create new tab"),
            Line::from("  alt-h/l        switch to prev / next tab"),
            Line::from(""),
            Line::from(vec![Span::styled("search", Style::default().fg(theme::VIOLET).bold())]),
            Line::from("  ctrl-s         search wikipedia (opens new tab)"),
            Line::from("  i              edit search query in current tab"),
            Line::from("  /              in-page text search"),
            Line::from("  n/N            jump to next / prev search match"),
            Line::from(""),
            Line::from(vec![Span::styled("general", Style::default().fg(theme::VIOLET).bold())]),
            Line::from("  ?              toggle this help popup"),
            Line::from("  q              quit wiki-tui"),
        ];

        let help_block = Block::bordered()
            .border_style(Style::default().fg(theme::PINK))
            .title(ratatui::widgets::block::Title::from(" keybindings ").alignment(ratatui::layout::Alignment::Center))
            .title(
                ratatui::widgets::block::Title::from(Span::styled(
                    " esc to close ",
                    Style::default().fg(theme::GREY).italic(),
                ))
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(ratatui::layout::Alignment::Right),
            );

        let help_paragraph = Paragraph::new(help_text).block(help_block);
        f.render_widget(help_paragraph, area);
    }

    // search popup
    if app.input_mode == InputMode::Search {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Length(3), // 1 character tall
                Constraint::Min(0),
            ])
            .split(size);

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(36),
                Constraint::Percentage(28),
                Constraint::Percentage(36),
            ])
            .split(popup_layout[1])[1];

        f.render_widget(Clear, area);

        let search_block = Block::bordered()
            .border_style(Style::default().fg(theme::BEIGE))
            .title(
                ratatui::widgets::block::Title::from(" search wikipedia ")
                    .alignment(ratatui::layout::Alignment::Left),
            );

        let input_text = Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme::BEIGE).bold()),
            Span::styled(&app.search_input, Style::default().fg(theme::FG).bold()),
            Span::styled("_", Style::default().fg(theme::BEIGE).bold()),
        ]);

        let search_paragraph = Paragraph::new(input_text).block(search_block);
        f.render_widget(search_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
