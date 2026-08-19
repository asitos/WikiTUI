use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{block::Title, Block},
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn create_modal_block(icon: &str, title: &str, border_color: Color) -> Block<'static> {
    let top_title = if icon.is_empty() {
        format!(" {} ", title)
    } else {
        format!(" {} {} ", icon, title)
    };

    Block::bordered()
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG))
        .title(
            Title::from(Span::styled(
                top_title,
                Style::default().fg(border_color).bold(),
            ))
            .alignment(Alignment::Center),
        )
}
