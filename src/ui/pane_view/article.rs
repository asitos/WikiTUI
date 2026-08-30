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

    let has_selected_link = pane
        .selected_link_idx
        .and_then(|idx| parsed_doc.links.get(idx))
        .is_some_and(|link| {
            link.span_indices
                .iter()
                .any(|&(line_idx, _)| line_idx >= view_start && line_idx < view_end)
        });

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

    if !has_underline && !has_selected_link && !has_search_matches {
        let borrowed_lines: Vec<Line<'_>> = parsed_doc.lines[view_start..view_end]
            .iter()
            .map(|line| {
                let spans: Vec<Span<'_>> = line
                    .spans
                    .iter()
                    .map(|s| Span::styled(s.content.as_ref(), s.style))
                    .collect();
                let mut l = Line::from(spans);
                l.alignment = line.alignment;
                l
            })
            .collect();
        let paragraph = Paragraph::new(borrowed_lines).block(block);
        f.render_widget(paragraph, rect);
    } else {
        let mut rendered_lines: Vec<Line<'static>> = parsed_doc.lines[view_start..view_end].to_vec();

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
                    let is_active = selected_match
                        .is_some_and(|sm| sm.line_idx == m.line_idx && sm.char_offset == m.char_offset);
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
    }

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

pub fn apply_search_highlights_to_line(
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
