use super::utils::render_modal_frame;
use crate::app::Pane;
use crate::parser::ParsedDocument;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_toc_modal(
    f: &mut Frame,
    pane: &Pane,
    parsed_doc: &ParsedDocument,
    container_rect: Rect,
    show_numbers: bool,
    rounded: bool,
    show_icons: bool,
) {
    let icon = if show_icons { "≡" } else { "" };
    let (toc_area, toc_block) = render_modal_frame(
        f,
        container_rect,
        60,
        60,
        icon,
        "contents",
        theme::LIME,
        rounded,
    );

    let current_scroll = pane.scroll_offset;
    let active_heading_idx = parsed_doc
        .headings
        .iter()
        .rposition(|h| h.line_idx <= current_scroll)
        .unwrap_or(0);

    let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

    let section_numbers = if show_numbers {
        compute_section_numbers(&parsed_doc.headings)
    } else {
        Vec::new()
    };

    let mut toc_lines = Vec::new();
    for (idx, h) in parsed_doc.headings.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let indent_len = ((h.level.saturating_sub(1)) * 2) as usize;
        let indent = " ".repeat(indent_len);
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default().fg(theme::LIME).bold()
        } else {
            Style::default().fg(theme::FG)
        };

        let num_str = section_numbers.get(idx).map(|s| s.as_str()).unwrap_or("");
        let num_len = num_str.len();

        let avail_w = (toc_area.width as usize).saturating_sub(6 + indent_len + num_len);
        let truncated_title = if h.title.chars().count() > avail_w && avail_w > 3 {
            let byte_end = h
                .title
                .char_indices()
                .nth(avail_w.saturating_sub(3))
                .map(|(i, _)| i)
                .unwrap_or(h.title.len());
            format!("{}...", &h.title[..byte_end])
        } else {
            h.title.clone()
        };

        let num_style = if is_selected {
            Style::default().fg(theme::LIME)
        } else {
            Style::default().fg(theme::GREY)
        };

        let mut spans = vec![Span::styled(prefix, style), Span::raw(indent)];
        if !num_str.is_empty() {
            spans.push(Span::styled(num_str.to_string(), num_style));
        }
        spans.push(Span::styled(truncated_title, style));

        toc_lines.push(Line::from(spans));
    }

    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = selected_idx.saturating_sub(visible_rows / 2);

    let toc_p = Paragraph::new(toc_lines)
        .block(toc_block)
        .scroll((toc_scroll as u16, 0));
    f.render_widget(toc_p, toc_area);
}

fn compute_section_numbers(headings: &[crate::parser::Heading]) -> Vec<String> {
    if headings.is_empty() {
        return Vec::new();
    }
    let min_level = headings.iter().map(|h| h.level).min().unwrap_or(1);
    let mut counters: Vec<usize> = Vec::new();
    let mut results = Vec::new();

    for h in headings {
        let depth = (h.level.saturating_sub(min_level)) as usize;
        if depth >= counters.len() {
            counters.resize(depth + 1, 0);
        } else {
            counters.truncate(depth + 1);
        }
        counters[depth] += 1;

        if depth == 0 {
            results.push(format!("{}. ", counters[0]));
        } else {
            let s = counters
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(".");
            results.push(format!("{} ", s));
        }
    }
    results
}
