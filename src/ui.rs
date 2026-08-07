use crate::app::{App, InputMode, PaneContent};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::{Line, Span, Style, Stylize},
    widgets::{Block, Paragraph},
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

    // tab bar
    let mut tab_spans = Vec::new();
    for (idx, tab) in app.tabs.iter().enumerate() {
        let name = tab.name.to_lowercase();
        if idx == app.active_tab_idx {
            tab_spans.push(Span::styled(
                format!(" [ {} ] ", name),
                Style::default().fg(theme::LIME).bold(),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("  {}  ", name),
                Style::default().fg(theme::DARK_GREY),
            ));
        }
    }
    let tab_bar_paragraph = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, tab_bar_area);

    // status bar
    let status_text = match app.input_mode {
        // for when in search mode
        InputMode::Search => format!(
            " search: {}_ | press enter to submit, esc to cancel ",
            app.search_input
        ),
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
        // normally
        InputMode::Normal => {
            "ctrl-s: search | /: local search | ]/[: headings | g/G: top/bottom | f/b: page | tab/backtab: links | enter/t/s/v: open (tab/split) | ctrl-w s/v: split | alt-h/l: tabs | q: quit".to_string()
        }
    };
    let status_style = match app.input_mode {
        InputMode::Search => Style::default().fg(theme::BEIGE).bold(),
        InputMode::LocalSearch => Style::default().fg(theme::YELLOW).bold(),
        InputMode::Normal => Style::default().fg(theme::GREY),
    };
    let status_paragraph = Paragraph::new(status_text).style(status_style);
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
            PaneContent::Empty => format!(" pane {} ", pane.id),
            PaneContent::SearchResults { query, .. } => {
                format!(" pane {} - search: {} ", pane.id, query.to_lowercase())
            }
            PaneContent::ArticleText { title, .. } => {
                format!(" pane {} - {} ", pane.id, title.to_lowercase())
            }
            PaneContent::Error(_) => format!(" pane {} - error ", pane.id),
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
                let empty_p = Paragraph::new("press 'ctrl-s' to search wikipedia")
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
                                // focused link color
                                .bg(theme::VIOLET)
                                .fg(theme::FG)
                                .bold();
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
}
