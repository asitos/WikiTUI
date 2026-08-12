use crate::app::{App, PaneContent};
use crate::theme;
use crate::ui::modals::render_toc_modal;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
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
        rect.width.saturating_sub(2) as usize
    } else {
        rect.width.saturating_sub(4) as usize
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
        Block::default().padding(Padding::horizontal(1))
    } else {
        Block::bordered()
            .border_style(Style::default().fg(border_color))
            .title(title)
            .padding(Padding::horizontal(1))
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

            if let Some(link) = pane
                .selected_link_idx
                .and_then(|idx| parsed_doc.links.get(idx))
            {
                for &(line_idx, span_idx) in &link.span_indices {
                    if let Some(line) = rendered_lines.get_mut(line_idx) {
                        if let Some(span) = line.spans.get_mut(span_idx) {
                            span.style = Style::default()
                                .fg(theme::VIOLET)
                                .bold()
                                .add_modifier(Modifier::UNDERLINED);
                        }
                    }
                }
            }

            // highlight local search matches
            let query = pane.local_search_query.to_lowercase();
            if !query.trim().is_empty() {
                let active_match = pane
                    .selected_match_idx
                    .and_then(|idx| pane.local_matches.get(idx));

                for (line_idx, line) in rendered_lines.iter_mut().enumerate() {
                    let full_line_text: String =
                        line.spans.iter().map(|s| s.content.as_ref()).collect();
                    let full_lower = full_line_text.to_lowercase();

                    if full_lower.contains(&query) {
                        let is_active_line = active_match.is_some_and(|m| m.line_idx == line_idx);
                        let bg_color = if is_active_line {
                            theme::YELLOW
                        } else {
                            theme::BEIGE
                        };

                        let mut match_ranges = Vec::new();
                        let mut start = 0;
                        while let Some(pos) = full_lower[start..].find(&query) {
                            let match_start = start + pos;
                            let match_end = match_start + query.len();
                            match_ranges.push((match_start, match_end));
                            start = match_end.max(start + 1);
                        }

                        let mut new_spans = Vec::new();
                        let mut current_global_offset = 0;

                        for span in &line.spans {
                            let text = &span.content;
                            let span_len = text.len();
                            let span_start = current_global_offset;
                            let span_end = span_start + span_len;

                            let mut text_cursor = 0;
                            while text_cursor < span_len {
                                let global_pos = span_start + text_cursor;
                                let active_range =
                                    match_ranges.iter().find(|&&(r_start, r_end)| {
                                        global_pos >= r_start && global_pos < r_end
                                    });

                                if let Some(&(_, r_end)) = active_range {
                                    let match_end_in_span = (r_end - span_start).min(span_len);
                                    let slice = &text[text_cursor..match_end_in_span];
                                    new_spans.push(Span::styled(
                                        slice.to_string(),
                                        Style::default().bg(bg_color).fg(theme::BG).bold(),
                                    ));
                                    text_cursor = match_end_in_span;
                                } else {
                                    let next_match_start = match_ranges
                                        .iter()
                                        .map(|&(r_start, _)| r_start)
                                        .filter(|&r_start| {
                                            r_start > global_pos && r_start < span_end
                                        })
                                        .min()
                                        .unwrap_or(span_end);

                                    let unmatch_end_in_span = next_match_start - span_start;
                                    let slice = &text[text_cursor..unmatch_end_in_span];
                                    new_spans.push(Span::styled(slice.to_string(), span.style));
                                    text_cursor = unmatch_end_in_span;
                                }
                            }

                            current_global_offset = span_end;
                        }

                        line.spans = new_spans;
                    }
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
