use super::utils::{centered_rect, render_modal_frame_at};
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

pub fn compute_toc_modal_area(container_rect: Rect) -> Rect {
    centered_rect(60, 60, container_rect)
}

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
    let toc_area = compute_toc_modal_area(container_rect);
    let toc_block = render_modal_frame_at(f, toc_area, icon, "contents", theme::LIME, rounded);

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

        let num_str = section_numbers.get(idx).map(|s| s.as_str()).unwrap_or("");
        let num_len = num_str.len();

        let avail_w = (toc_area.width as usize).saturating_sub(6 + indent_len + num_len);
        let truncated_title = crate::ui::truncate_to_width(&h.title, avail_w);

        let prefix = if is_selected { " ▶ " } else { "   " };
        let text_style = if is_selected {
            Style::default().fg(theme::YELLOW).bold()
        } else {
            Style::default().fg(theme::FG)
        };
        let num_style = if is_selected {
            Style::default().fg(theme::YELLOW).dim()
        } else {
            Style::default().fg(theme::GREY)
        };

        let mut spans = vec![
            Span::styled(prefix, Style::default().fg(theme::LIME)),
            Span::raw(indent),
        ];

        if !num_str.is_empty() {
            spans.push(Span::styled(num_str.to_string(), num_style));
        }

        spans.push(Span::styled(truncated_title, text_style));

        toc_lines.push(Line::from(spans));
    }

    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = compute_toc_scroll(selected_idx, visible_rows, parsed_doc.headings.len());

    let toc_p = Paragraph::new(toc_lines)
        .block(toc_block)
        .scroll((toc_scroll as u16, 0));
    f.render_widget(toc_p, toc_area);
}

pub fn compute_toc_scroll(
    selected_idx: usize,
    visible_rows: usize,
    total_headings: usize,
) -> usize {
    if total_headings <= visible_rows || visible_rows == 0 {
        0
    } else {
        selected_idx
            .saturating_sub(visible_rows / 2)
            .min(total_headings.saturating_sub(visible_rows))
    }
}

pub fn get_toc_heading_at(
    parsed_doc: &ParsedDocument,
    selected_idx: usize,
    toc_area: Rect,
    target_y: u16,
) -> Option<usize> {
    if target_y <= toc_area.y || target_y >= toc_area.y + toc_area.height.saturating_sub(1) {
        return None;
    }
    let row_offset = (target_y - (toc_area.y + 1)) as usize;
    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = compute_toc_scroll(selected_idx, visible_rows, parsed_doc.headings.len());
    let clicked_idx = toc_scroll + row_offset;
    if clicked_idx < parsed_doc.headings.len() {
        Some(clicked_idx)
    } else {
        None
    }
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
