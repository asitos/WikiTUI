use crate::theme;
use ratatui::{
    layout::{Margin, Rect},
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

#[allow(clippy::too_many_arguments)]
pub fn render_scroll_indicator(
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
