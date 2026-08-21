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

pub fn create_modal_block(
    icon: &str,
    title: &str,
    border_color: Color,
    rounded: bool,
) -> Block<'static> {
    let top_title = if icon.is_empty() {
        format!(" {} ", title)
    } else {
        format!(" {} {} ", icon, title)
    };

    let border_type = if rounded {
        ratatui::widgets::BorderType::Rounded
    } else {
        ratatui::widgets::BorderType::Plain
    };

    Block::bordered()
        .border_type(border_type)
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

#[allow(clippy::too_many_arguments)]
pub fn render_modal_frame(
    f: &mut ratatui::Frame,
    size: Rect,
    percent_x: u16,
    percent_y: u16,
    icon: &str,
    title: &str,
    border_color: Color,
    rounded: bool,
) -> (Rect, Block<'static>) {
    let area = centered_rect(percent_x, percent_y, size);
    f.render_widget(ratatui::widgets::Clear, area);
    let block = create_modal_block(icon, title, border_color, rounded);
    (area, block)
}

#[allow(clippy::too_many_arguments)]
pub fn render_modal_container(
    f: &mut ratatui::Frame,
    size: Rect,
    percent_x: u16,
    percent_y: u16,
    icon: &str,
    title: &str,
    border_color: Color,
    rounded: bool,
) -> Rect {
    let (area, block) = render_modal_frame(
        f,
        size,
        percent_x,
        percent_y,
        icon,
        title,
        border_color,
        rounded,
    );
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

pub fn create_checkbox_line(
    label: &str,
    is_focused: bool,
    is_checked: bool,
    suffix: Option<&str>,
    cursor_color: Color,
) -> ratatui::text::Line<'static> {
    let cursor_str = if is_focused { " ▶ " } else { "   " };
    let check_str = if is_checked { "[x] " } else { "[ ] " };

    let item_style = if is_focused {
        Style::default().fg(theme::YELLOW).bold()
    } else if is_checked {
        Style::default().fg(theme::LIME)
    } else {
        Style::default().fg(theme::FG)
    };

    let check_style = if is_checked {
        Style::default().fg(theme::LIME).bold()
    } else {
        Style::default().fg(theme::GREY)
    };

    let mut spans = vec![
        Span::styled(cursor_str, Style::default().fg(cursor_color).bold()),
        Span::styled(check_str, check_style),
        Span::styled(label.to_string(), item_style),
    ];

    if let Some(suf) = suffix {
        spans.push(Span::styled(
            suf.to_string(),
            Style::default().fg(theme::GREY),
        ));
    }

    ratatui::text::Line::from(spans)
}

pub fn create_selectable_line(
    label: &str,
    is_selected: bool,
    is_active: bool,
    cursor_color: Color,
    suffix: Option<&str>,
) -> ratatui::text::Line<'static> {
    let prefix = if is_selected { " ▶ " } else { "   " };
    let style = if is_selected && is_active {
        Style::default().fg(theme::YELLOW).bold()
    } else if is_selected {
        Style::default().fg(theme::VIOLET).bold()
    } else {
        Style::default().fg(theme::FG)
    };

    let mut spans = vec![
        Span::styled(prefix, Style::default().fg(cursor_color)),
        Span::styled(label.to_string(), style),
    ];

    if let Some(suf) = suffix {
        spans.push(Span::styled(
            suf.to_string(),
            Style::default().fg(theme::GREY),
        ));
    }

    ratatui::text::Line::from(spans)
}
