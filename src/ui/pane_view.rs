use crate::app::{App, PaneContent};
use crate::theme;
use crate::ui::modals::render_toc_modal;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn render_single_active_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let active_tab_idx = app.active_tab_idx;
    let active_pane_idx = app.tabs[active_tab_idx].active_pane_idx;
    render_pane_at(f, app, active_tab_idx, active_pane_idx, area, true);
}

pub fn render_panes(f: &mut Frame, app: &mut App, main_area: Rect) {
    let active_tab_idx = app.active_tab_idx;
    let rects = app.tabs[active_tab_idx]
        .layout_root
        .compute_rects(main_area);
    let active_pane_idx = app.tabs[active_tab_idx].active_pane_idx;

    for (pane_idx, rect) in rects {
        let is_active = pane_idx == active_pane_idx;
        render_pane_at(f, app, active_tab_idx, pane_idx, rect, is_active);
    }
}

fn render_pane_at(
    f: &mut Frame,
    app: &mut App,
    tab_idx: usize,
    pane_idx: usize,
    rect: Rect,
    is_active: bool,
) {
    let content_width = if app.zen_mode {
        rect.width as usize
    } else {
        rect.width.saturating_sub(2) as usize
    };

    let pane = &mut app.tabs[tab_idx].panes[pane_idx];
    pane.ensure_parsed_width(content_width);
    pane.viewport_height = if app.zen_mode {
        rect.height as usize
    } else {
        rect.height.saturating_sub(2) as usize
    };

    let border_color = match &pane.content {
        PaneContent::SearchResults { .. } => {
            if is_active {
                theme::YELLOW
            } else {
                theme::DARK_GREY
            }
        }
        _ => {
            if is_active {
                theme::PINK
            } else {
                theme::DARK_GREY
            }
        }
    };

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

    let block = if app.zen_mode {
        Block::default()
    } else {
        Block::bordered()
            .border_style(Style::default().fg(border_color))
            .title(title)
    };

    if pane.is_loading {
        let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            "loading wikipedia data...",
            Style::default().fg(theme::YELLOW).bold(),
        )));
        let loading_p = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(loading_p, rect);
        return;
    }

    match &pane.content {
        PaneContent::Empty => {
            let empty_p = Paragraph::new("")
                .fg(theme::GREY)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(empty_p, rect);
        }
        PaneContent::SearchResults { items, .. } => {
            if items.is_empty() {
                let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
                let mut lines = Vec::new();
                for _ in 0..vertical_offset {
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    "no search results found",
                    Style::default().fg(theme::RED).bold(),
                )));
                let no_res_p = Paragraph::new(lines)
                    .block(block)
                    .alignment(Alignment::Center);
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
                            .fg(theme::VIOLET)
                            .bold()
                            .add_modifier(Modifier::UNDERLINED);
                    }
                }
            }

            // highlight local search matches
            let query = pane.local_search_query.trim().to_lowercase();
            if !query.is_empty() {
                let active_match = pane
                    .selected_match_idx
                    .and_then(|idx| pane.local_matches.get(idx));

                for (line_idx, line) in rendered_lines.iter_mut().enumerate() {
                    let mut new_spans = Vec::new();
                    for (span_idx, span) in line.spans.iter().enumerate() {
                        let text = &span.content;
                        let text_lower = text.to_lowercase();

                        if text_lower.contains(&query) {
                            let is_active_span = active_match
                                .is_some_and(|m| m.line_idx == line_idx && m.span_idx == span_idx);
                            let bg_color = if is_active_span {
                                theme::YELLOW
                            } else {
                                theme::BEIGE
                            };

                            let mut start = 0;
                            while let Some(rel_pos) = text_lower[start..].find(&query) {
                                let match_start = start + rel_pos;
                                let match_end = match_start + query.len();

                                if match_start > start {
                                    new_spans.push(Span::styled(
                                        text[start..match_start].to_string(),
                                        span.style,
                                    ));
                                }

                                let matched_text = text[match_start..match_end].to_string();
                                new_spans.push(Span::styled(
                                    matched_text,
                                    Style::default().bg(bg_color).fg(theme::BG).bold(),
                                ));

                                start = match_end;
                            }

                            if start < text.len() {
                                new_spans.push(Span::styled(text[start..].to_string(), span.style));
                            }
                        } else {
                            new_spans.push(span.clone());
                        }
                    }
                    line.spans = new_spans;
                }
            }

            let paragraph = Paragraph::new(rendered_lines)
                .block(block)
                .scroll((pane.scroll_offset as u16, 0));
            f.render_widget(paragraph, rect);

            if is_active && pane.show_toc && !parsed_doc.headings.is_empty() {
                render_toc_modal(f, pane, parsed_doc, rect);
            }
        }
        PaneContent::Error(err_msg) => {
            let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
            let mut lines = Vec::new();
            for _ in 0..vertical_offset {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                format!("error: {}", err_msg),
                Style::default().fg(theme::RED).bold(),
            )));
            let err_p = Paragraph::new(lines)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(err_p, rect);
        }
    }
}
