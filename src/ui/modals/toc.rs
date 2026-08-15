use super::utils::centered_rect;
use crate::app::Pane;
use crate::parser::ParsedDocument;
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph},
    Frame,
};

pub fn render_toc_modal(
    f: &mut Frame,
    pane: &Pane,
    parsed_doc: &ParsedDocument,
    container_rect: Rect,
) {
    let toc_area = centered_rect(60, 60, container_rect);
    f.render_widget(Clear, toc_area);

    let toc_block = Block::bordered()
        .border_style(Style::default().fg(theme::LIME))
        .title(" contents ")
        .title(
            Title::from(" enter: jump | o: close ")
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Right),
        );

    let current_scroll = pane.scroll_offset;
    let active_heading_idx = parsed_doc
        .headings
        .iter()
        .rposition(|h| h.line_idx <= current_scroll)
        .unwrap_or(0);

    let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

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

        let avail_w = (toc_area.width as usize).saturating_sub(6 + indent_len);
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

        toc_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(indent),
            Span::styled(truncated_title, style),
        ]));
    }

    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = selected_idx.saturating_sub(visible_rows / 2);

    let toc_p = Paragraph::new(toc_lines)
        .block(toc_block)
        .scroll((toc_scroll as u16, 0));
    f.render_widget(toc_p, toc_area);
}
