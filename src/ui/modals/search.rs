use crate::app::App;
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph},
    Frame,
};

pub fn render_search_modal(f: &mut Frame, app: &App, size: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(size);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(28),
            Constraint::Percentage(36),
        ])
        .split(popup_layout[1])[1];

    f.render_widget(Clear, area);

    let search_block = Block::bordered()
        .border_style(Style::default().fg(theme::BEIGE))
        .title(Title::from(" search wikipedia ").alignment(Alignment::Left));

    let visible_width = (area.width as usize).saturating_sub(6);
    let chars: Vec<char> = app.search_input.chars().collect();
    let total_len = chars.len();
    let cursor_pos = app.search_cursor_pos.min(total_len);

    let mut scroll_offset = 0;
    if cursor_pos >= visible_width && visible_width > 0 {
        scroll_offset = cursor_pos + 1 - visible_width;
    }

    let end_idx = (scroll_offset + visible_width).min(total_len);
    let visible_chars = &chars[scroll_offset..end_idx];
    let rel_cursor_pos = cursor_pos.saturating_sub(scroll_offset);

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " > ",
        Style::default().fg(theme::BEIGE).bold(),
    ));

    for (i, &ch) in visible_chars.iter().enumerate() {
        if i == rel_cursor_pos {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(theme::BEIGE).fg(theme::BG).bold(),
            ));
        } else {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme::FG).bold(),
            ));
        }
    }

    if rel_cursor_pos >= visible_chars.len() {
        spans.push(Span::styled("_", Style::default().fg(theme::BEIGE).bold()));
    }

    let input_text = Line::from(spans);
    let search_paragraph = Paragraph::new(input_text).block(search_block);
    f.render_widget(search_paragraph, area);
}
