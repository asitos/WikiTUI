use super::scrollbar::render_scroll_indicator;
use crate::app::App;
use crate::theme;
use crate::ui::modals::render_toc_modal;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

#[allow(clippy::too_many_arguments)]
pub fn render_article_pane(
    f: &mut Frame,
    app: &mut App,
    tab_idx: usize,
    pane_idx: usize,
    rect: Rect,
    block: Block,
    border_color: ratatui::style::Color,
    is_active: bool,
) {
    let pane = &app.tabs[tab_idx].panes[pane_idx];
    let crate::app::PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
        return;
    };

    let view_start = pane.scroll_offset.min(parsed_doc.lines.len());
    let view_len =
        (pane.viewport_height + 2).min(parsed_doc.lines.len().saturating_sub(view_start));
    let view_end = view_start + view_len;

    let has_underline = if app.config.reader.underline_links {
        let first_link_idx = parsed_doc.links.partition_point(|link| {
            link.span_indices
                .last()
                .map(|&(l, _)| l < view_start)
                .unwrap_or(true)
        });
        parsed_doc.links[first_link_idx..]
            .iter()
            .take_while(|l| {
                l.span_indices
                    .first()
                    .is_some_and(|&(first_line, _)| first_line < view_end)
            })
            .any(|l| !l.is_citation())
    } else {
        false
    };

    let selected_link = pane
        .selected_link_idx
        .and_then(|idx| parsed_doc.links.get(idx));

    let has_search_matches =
        if !pane.local_matches.is_empty() && !pane.local_search_query.trim().is_empty() {
            let first_match_idx = pane
                .local_matches
                .partition_point(|m| m.line_idx < view_start);
            first_match_idx < pane.local_matches.len()
                && pane.local_matches[first_match_idx].line_idx < view_end
        } else {
            false
        };

    let mut rendered_lines: Vec<Line<'_>> = Vec::with_capacity(view_len);

    let first_link_idx = if has_underline {
        parsed_doc.links.partition_point(|link| {
            link.span_indices
                .last()
                .map(|&(l, _)| l < view_start)
                .unwrap_or(true)
        })
    } else {
        0
    };

    let query_len = pane.local_search_query.len();
    let mut match_ptr = if has_search_matches {
        pane.local_matches
            .partition_point(|m| m.line_idx < view_start)
    } else {
        0
    };
    let selected_match = pane
        .selected_match_idx
        .and_then(|idx| pane.local_matches.get(idx));

    for (local_idx, orig_line) in parsed_doc.lines[view_start..view_end].iter().enumerate() {
        let line_idx = view_start + local_idx;

        let mut spans: Vec<Span<'_>> = orig_line
            .spans
            .iter()
            .map(|s| Span::styled(s.content.as_ref(), s.style))
            .collect();

        if has_underline {
            for link in &parsed_doc.links[first_link_idx..] {
                let Some(&(first_line, _)) = link.span_indices.first() else {
                    continue;
                };
                if first_line > line_idx {
                    break;
                }
                if link.is_citation() {
                    continue;
                }
                for &(l_idx, span_idx) in &link.span_indices {
                    if l_idx == line_idx {
                        if let Some(span) = spans.get_mut(span_idx) {
                            span.style = span.style.add_modifier(Modifier::UNDERLINED);
                        }
                    }
                }
            }
        }

        if let Some(link) = selected_link {
            for &(l_idx, span_idx) in &link.span_indices {
                if l_idx == line_idx {
                    if let Some(span) = spans.get_mut(span_idx) {
                        span.style = Style::default()
                            .fg(theme::VIOLET)
                            .bold()
                            .add_modifier(Modifier::UNDERLINED);
                    }
                }
            }
        }

        if has_search_matches {
            let mut line_matches = Vec::new();
            while match_ptr < pane.local_matches.len()
                && pane.local_matches[match_ptr].line_idx == line_idx
            {
                let m = &pane.local_matches[match_ptr];
                let is_active = selected_match
                    .is_some_and(|sm| sm.line_idx == m.line_idx && sm.char_offset == m.char_offset);
                line_matches.push((m.char_offset, m.char_offset + query_len, is_active));
                match_ptr += 1;
            }

            if !line_matches.is_empty() {
                spans = build_search_highlighted_spans(&spans, &line_matches);
            }
        }

        if let Some(selection) = &pane.text_selection {
            if selection.contains_line(line_idx) {
                let (start, end) = selection.normalized();
                let line_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                let from = if line_idx == start.0 { start.1.min(line_len) } else { 0 };
                let to = if line_idx == end.0 { end.1.min(line_len) } else { line_len };
                if from < to {
                    spans = build_selection_highlighted_spans(&spans, from, to);
                }
            }
        }

        let mut line = Line::from(spans);
        line.alignment = orig_line.alignment;
        rendered_lines.push(line);
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

pub fn build_search_highlighted_spans<'a>(
    spans: &[Span<'a>],
    line_matches: &[(usize, usize, bool)],
) -> Vec<Span<'a>> {
    if line_matches.is_empty() {
        return spans.to_vec();
    }

    let mut new_spans = Vec::with_capacity(spans.len() + line_matches.len() * 2);
    let mut global_offset = 0;

    for span in spans {
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
                let unmatch_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => {
                        Span::styled(&s[text_cursor..rel_match_start], span.style)
                    }
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[text_cursor..rel_match_start].to_string(), span.style)
                    }
                };
                new_spans.push(unmatch_span);
                text_cursor = rel_match_start;
            }

            if rel_match_end > text_cursor && rel_match_end <= span_len {
                let bg_color = if is_active {
                    theme::YELLOW
                } else {
                    theme::BEIGE
                };
                let match_style = Style::default().bg(bg_color).fg(theme::BG).bold();
                let match_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => {
                        Span::styled(&s[text_cursor..rel_match_end], match_style)
                    }
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[text_cursor..rel_match_end].to_string(), match_style)
                    }
                };
                new_spans.push(match_span);
                text_cursor = rel_match_end;
            }
        }

        if text_cursor < span_len {
            let trailing_span = match &span.content {
                std::borrow::Cow::Borrowed(s) => Span::styled(&s[text_cursor..], span.style),
                std::borrow::Cow::Owned(s) => {
                    Span::styled(s[text_cursor..].to_string(), span.style)
                }
            };
            new_spans.push(trailing_span);
        }

        global_offset = span_end;
    }

    new_spans
}

fn build_selection_highlighted_spans<'a>(
    spans: &[Span<'a>],
    sel_start: usize,
    sel_end: usize,
) -> Vec<Span<'a>> {
    let mut new_spans = Vec::new();
    let mut global_offset = 0;

    for span in spans {
        let span_len = span.content.chars().count();
        let span_start = global_offset;
        let span_end = span_start + span_len;

        if sel_end <= span_start || sel_start >= span_end {
            new_spans.push(span.clone());
        } else {
            let rel_start = sel_start.saturating_sub(span_start).min(span_len);
            let rel_end = sel_end.saturating_sub(span_start).min(span_len);

            let chars: Vec<char> = span.content.chars().collect();

            if rel_start > 0 {
                let prefix: String = chars[0..rel_start].iter().collect();
                new_spans.push(Span::styled(prefix, span.style));
            }

            if rel_end > rel_start {
                let selected: String = chars[rel_start..rel_end].iter().collect();
                let sel_style = Style::default().bg(theme::PINK).fg(theme::BG).bold();
                new_spans.push(Span::styled(selected, sel_style));
            }

            if rel_end < span_len {
                let suffix: String = chars[rel_end..].iter().collect();
                new_spans.push(Span::styled(suffix, span.style));
            }
        }

        global_offset = span_end;
    }

    new_spans
}

pub fn get_link_at_coord(
    parsed_doc: &crate::parser::ParsedDocument,
    scroll_offset: usize,
    pane_rect: Rect,
    col: u16,
    row: u16,
) -> Option<usize> {
    if pane_rect.width < 3 || pane_rect.height < 3 {
        return None;
    }
    let inner_x = pane_rect.x + 1;
    let inner_y = pane_rect.y + 1;
    let inner_w = pane_rect.width.saturating_sub(2);
    let inner_h = pane_rect.height.saturating_sub(2);

    if col < inner_x || col >= inner_x + inner_w || row < inner_y || row >= inner_y + inner_h {
        return None;
    }

    let row_in_pane = (row - inner_y) as usize;
    let line_idx = scroll_offset + row_in_pane;
    let line = parsed_doc.lines.get(line_idx)?;

    let target_x = (col - inner_x) as usize;
    let mut cur_x = 0;
    let mut target_span_idx = None;

    for (span_idx, span) in line.spans.iter().enumerate() {
        let span_w = unicode_width::UnicodeWidthStr::width(span.content.as_ref());
        if target_x >= cur_x && target_x < cur_x + span_w {
            target_span_idx = Some(span_idx);
            break;
        }
        cur_x += span_w;
    }

    let span_idx = target_span_idx?;

    parsed_doc
        .links
        .iter()
        .position(|link| link.span_indices.contains(&(line_idx, span_idx)))
}
