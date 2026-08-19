use crate::app::{App, PaneContent};
use crate::theme;
use crate::ui::modals::render_toc_modal;
use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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

    let show_footnotes = app.config.reader.show_footnotes;
    let show_external_links = app.config.reader.show_external_links;
    let pane = &mut app.tabs[tab_idx].panes[pane_idx];
    pane.ensure_parsed_width(content_width, show_footnotes, show_external_links);
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

    let border_type = if app.config.ui.rounded_borders {
        ratatui::widgets::BorderType::Rounded
    } else {
        ratatui::widgets::BorderType::Plain
    };

    let block = if app.zen_mode {
        Block::default().padding(Padding::horizontal(1))
    } else {
        Block::bordered()
            .border_type(border_type)
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

        let spinner = crate::ui::current_spinner_frame();

        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::LIME).bold(),
            ),
            Span::styled(
                "loading wikipedia data...",
                Style::default().fg(theme::BEIGE).bold(),
            ),
        ]));
        let loading_p = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(loading_p, rect);
        return;
    }

    match &pane.content {
        PaneContent::Empty => {
            crate::ui::launch_screen::render_launch_screen(f, app, rect, block);
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
                let inner_width = (rect.width as usize).saturating_sub(4).max(20);

                for (i, item) in items.iter().enumerate() {
                    let is_selected = i == pane.selected_idx;
                    let title_lower = item.title.to_lowercase();
                    let snippet_lower = item.snippet.to_lowercase();

                    if is_selected {
                        let badge_str = format!(" {} ", i + 1);
                        let badge_w = unicode_width::UnicodeWidthStr::width(badge_str.as_str());
                        let title_w = unicode_width::UnicodeWidthStr::width(title_lower.as_str());
                        let pad_1 = inner_width.saturating_sub(badge_w + 1 + title_w);

                        lines.push(Line::from(vec![
                            Span::styled(
                                badge_str,
                                Style::default().bg(theme::LIME).fg(theme::BG).bold(),
                            ),
                            Span::styled(" ", Style::default().bg(theme::LIGHT_BG)),
                            Span::styled(
                                title_lower,
                                Style::default().bg(theme::LIGHT_BG).fg(theme::LIME).bold(),
                            ),
                            Span::styled(" ".repeat(pad_1), Style::default().bg(theme::LIGHT_BG)),
                        ]));

                        if !snippet_lower.is_empty() {
                            let wrap_w = inner_width.saturating_sub(3).max(10);
                            for s_line in wrap_text(&snippet_lower, wrap_w) {
                                let s_w = unicode_width::UnicodeWidthStr::width(s_line.as_str());
                                let pad_s = inner_width.saturating_sub(3 + s_w);
                                lines.push(Line::from(vec![
                                    Span::styled("   ", Style::default().bg(theme::LIGHT_BG)),
                                    Span::styled(
                                        s_line,
                                        Style::default().bg(theme::LIGHT_BG).fg(theme::GREY),
                                    ),
                                    Span::styled(
                                        " ".repeat(pad_s),
                                        Style::default().bg(theme::LIGHT_BG),
                                    ),
                                ]));
                            }
                        }
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!(" {:>2} ", i + 1),
                                Style::default().fg(theme::DARK_GREY),
                            ),
                            Span::styled(title_lower, Style::default().fg(theme::FG).bold()),
                        ]));

                        if !snippet_lower.is_empty() {
                            let wrap_w = inner_width.saturating_sub(4).max(10);
                            for s_line in wrap_text(&snippet_lower, wrap_w) {
                                lines.push(Line::from(vec![
                                    Span::raw("    "),
                                    Span::styled(s_line, Style::default().fg(theme::GREY)),
                                ]));
                            }
                        }
                    }
                    lines.push(Line::from(""));
                }
                let total_lines = lines.len();
                let results_p = Paragraph::new(lines)
                    .block(block)
                    .scroll((pane.scroll_offset as u16, 0));
                f.render_widget(results_p, rect);

                render_scroll_indicator(
                    f,
                    rect,
                    total_lines,
                    pane.viewport_height,
                    pane.scroll_offset,
                    border_color,
                    is_active,
                    app.zen_mode,
                );
            }
        }
        PaneContent::ArticleText { parsed_doc, .. } => {
            let view_start = pane.scroll_offset;
            let view_len =
                (pane.viewport_height + 2).min(parsed_doc.lines.len().saturating_sub(view_start));
            let view_end = view_start + view_len;

            let mut rendered_lines: Vec<Line<'static>> = parsed_doc
                .lines
                .iter()
                .skip(view_start)
                .take(view_len)
                .cloned()
                .collect();

            if app.config.reader.underline_links {
                for link in &parsed_doc.links {
                    for &(line_idx, span_idx) in &link.span_indices {
                        if line_idx >= view_start && line_idx < view_end {
                            if let Some(line) = rendered_lines.get_mut(line_idx - view_start) {
                                if let Some(span) = line.spans.get_mut(span_idx) {
                                    span.style = span.style.add_modifier(Modifier::UNDERLINED);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(link) = pane
                .selected_link_idx
                .and_then(|idx| parsed_doc.links.get(idx))
            {
                for &(line_idx, span_idx) in &link.span_indices {
                    if line_idx >= view_start && line_idx < view_end {
                        if let Some(line) = rendered_lines.get_mut(line_idx - view_start) {
                            if let Some(span) = line.spans.get_mut(span_idx) {
                                span.style = Style::default()
                                    .fg(theme::VIOLET)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED);
                            }
                        }
                    }
                }
            }

            let query = pane.local_search_query.to_lowercase();
            if !query.trim().is_empty() {
                let active_match = pane
                    .selected_match_idx
                    .and_then(|idx| pane.local_matches.get(idx));

                for (local_idx, line) in rendered_lines.iter_mut().enumerate() {
                    let line_idx = view_start + local_idx;
                    let full_line_text: String =
                        line.spans.iter().map(|s| s.content.as_ref()).collect();
                    let full_lower = full_line_text.to_lowercase();

                    if full_lower.contains(&query) {
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

                                if let Some(&(r_start, r_end)) = active_range {
                                    let is_this_active = active_match.is_some_and(|m| {
                                        m.line_idx == line_idx && m.char_offset == r_start
                                    });
                                    let bg_color = if is_this_active {
                                        theme::YELLOW
                                    } else {
                                        theme::BEIGE
                                    };

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

            let paragraph = Paragraph::new(rendered_lines).block(block);
            f.render_widget(paragraph, rect);

            render_scroll_indicator(
                f,
                rect,
                parsed_doc.lines.len(),
                pane.viewport_height,
                pane.scroll_offset,
                border_color,
                is_active,
                app.zen_mode,
            );

            if is_active && pane.show_toc && !parsed_doc.headings.is_empty() {
                render_toc_modal(
                    f,
                    pane,
                    parsed_doc,
                    rect,
                    app.config.reader.toc_section_numbers,
                    app.config.ui.rounded_borders,
                );
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

#[allow(clippy::too_many_arguments)]
fn render_scroll_indicator(
    f: &mut Frame,
    rect: Rect,
    total_lines: usize,
    viewport_height: usize,
    scroll_offset: usize,
    border_color: ratatui::style::Color,
    is_active: bool,
    zen_mode: bool,
) {
    if !zen_mode && total_lines > viewport_height {
        let total_scrollable = total_lines.saturating_sub(viewport_height);
        let mut scrollbar_state = ScrollbarState::new(total_scrollable).position(scroll_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("┃")
            .track_style(Style::default().fg(theme::DARK_GREY))
            .thumb_style(Style::default().fg(if is_active {
                border_color
            } else {
                theme::DARK_GREY
            }));

        let scroll_area = rect.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        });
        f.render_stateful_widget(scrollbar, scroll_area, &mut scrollbar_state);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = unicode_width::UnicodeWidthStr::width(word);
        if current_line.is_empty() {
            current_line.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
