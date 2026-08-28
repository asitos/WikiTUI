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
    let heading_marker = app.config.reader.heading_marker;
    let code_line_numbers = app.config.reader.code_line_numbers;
    let pane = &mut app.tabs[tab_idx].panes[pane_idx];
    pane.ensure_parsed_width(
        content_width,
        show_footnotes,
        show_external_links,
        heading_marker,
        code_line_numbers,
    );
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

    let border_type = app.config.ui.border_type();

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
                let inner_width = (rect.width as usize).saturating_sub(4).max(20);
                let item_counts =
                    compute_search_result_lines_count(items, pane.selected_idx, inner_width);
                let total_lines: usize = item_counts.iter().sum();
                let view_start = pane.scroll_offset;
                let view_end = view_start + pane.viewport_height + 2;

                let mut lines = Vec::with_capacity(total_lines);
                let mut cur_line = 0;

                for (i, item) in items.iter().enumerate() {
                    let item_height = item_counts[i];
                    let item_start = cur_line;
                    let item_end = item_start + item_height;
                    cur_line = item_end;

                    if item_end <= view_start || item_start >= view_end {
                        for _ in 0..item_height {
                            lines.push(Line::from(""));
                        }
                        continue;
                    }

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
                    app.config.ui.scroll_indicator,
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
                let first_link_idx = parsed_doc.links.partition_point(|link| {
                    link.span_indices
                        .last()
                        .map(|&(l, _)| l < view_start)
                        .unwrap_or(true)
                });

                for link in &parsed_doc.links[first_link_idx..] {
                    let Some(&(first_line, _)) = link.span_indices.first() else {
                        continue;
                    };
                    if first_line >= view_end {
                        break;
                    }
                    if link.is_citation() {
                        continue;
                    }
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

            if !pane.local_matches.is_empty() && !pane.local_search_query.trim().is_empty() {
                let query_len = pane.local_search_query.len();
                let first_match_idx = pane
                    .local_matches
                    .partition_point(|m| m.line_idx < view_start);
                let selected_match = pane
                    .selected_match_idx
                    .and_then(|idx| pane.local_matches.get(idx));

                let mut match_ptr = first_match_idx;
                for (local_idx, line) in rendered_lines.iter_mut().enumerate() {
                    let line_idx = view_start + local_idx;
                    let mut line_matches = Vec::new();

                    while match_ptr < pane.local_matches.len()
                        && pane.local_matches[match_ptr].line_idx == line_idx
                    {
                        let m = &pane.local_matches[match_ptr];
                        let is_active = selected_match.is_some_and(|sm| {
                            sm.line_idx == m.line_idx && sm.char_offset == m.char_offset
                        });
                        line_matches.push((m.char_offset, m.char_offset + query_len, is_active));
                        match_ptr += 1;
                    }

                    if !line_matches.is_empty() {
                        apply_search_highlights_to_line(line, &line_matches);
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
                app.config.ui.scroll_indicator,
            );

            if is_active && pane.show_toc && !parsed_doc.headings.is_empty() {
                render_toc_modal(
                    f,
                    pane,
                    parsed_doc,
                    rect,
                    app.config.reader.toc_section_numbers,
                    app.config.ui.rounded_borders,
                    app.config.ui.icons,
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
    show_indicator: bool,
) {
    if show_indicator && !zen_mode && total_lines > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll_offset)
            .viewport_content_length(viewport_height);
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

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
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

pub fn count_wrapped_lines(text: &str, max_width: usize) -> usize {
    if text.trim().is_empty() {
        return 0;
    }
    let mut count = 1;
    let mut current_width = 0;
    for word in text.split_whitespace() {
        let word_width = unicode_width::UnicodeWidthStr::width(word);
        if current_width == 0 {
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            current_width += 1 + word_width;
        } else {
            count += 1;
            current_width = word_width;
        }
    }
    count
}

pub fn compute_search_result_lines_count(
    items: &[crate::api::SearchResultItem],
    selected_idx: usize,
    inner_width: usize,
) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let snippet_lines = if !item.snippet.is_empty() {
                let wrap_w = if i == selected_idx {
                    inner_width.saturating_sub(3).max(10)
                } else {
                    inner_width.saturating_sub(4).max(10)
                };
                count_wrapped_lines(&item.snippet, wrap_w)
            } else {
                0
            };
            1 + snippet_lines + 1
        })
        .collect()
}

pub fn get_search_result_at_line(
    items: &[crate::api::SearchResultItem],
    selected_idx: usize,
    inner_width: usize,
    target_line: usize,
) -> Option<usize> {
    let mut cur_line = 0;
    let counts = compute_search_result_lines_count(items, selected_idx, inner_width);
    for (i, count) in counts.iter().enumerate() {
        if target_line >= cur_line && target_line < cur_line + count {
            return Some(i);
        }
        cur_line += count;
    }
    None
}

fn apply_search_highlights_to_line(
    line: &mut Line<'static>,
    line_matches: &[(usize, usize, bool)],
) {
    if line_matches.is_empty() {
        return;
    }

    let mut new_spans = Vec::with_capacity(line.spans.len() + line_matches.len() * 2);
    let mut global_offset = 0;

    for span in &line.spans {
        let text = span.content.as_ref();
        let span_len = text.len();
        let span_start = global_offset;
        let span_end = span_start + span_len;

        let mut text_cursor = 0;

        for &(m_start, m_end, is_active) in line_matches {
            if m_end <= span_start || m_start >= span_end {
                continue;
            }

            let rel_match_start = m_start.saturating_sub(span_start).max(text_cursor);
            let rel_match_end = (m_end.saturating_sub(span_start)).min(span_len);

            if rel_match_start > text_cursor && rel_match_start <= span_len {
                let unmatch_slice = &text[text_cursor..rel_match_start];
                new_spans.push(Span::styled(unmatch_slice.to_string(), span.style));
                text_cursor = rel_match_start;
            }

            if rel_match_end > text_cursor && rel_match_end <= span_len {
                let match_slice = &text[text_cursor..rel_match_end];
                let bg_color = if is_active {
                    theme::YELLOW
                } else {
                    theme::BEIGE
                };
                new_spans.push(Span::styled(
                    match_slice.to_string(),
                    Style::default().bg(bg_color).fg(theme::BG).bold(),
                ));
                text_cursor = rel_match_end;
            }
        }

        if text_cursor < span_len {
            let trailing_slice = &text[text_cursor..];
            new_spans.push(Span::styled(trailing_slice.to_string(), span.style));
        }

        global_offset = span_end;
    }

    line.spans = new_spans;
}
