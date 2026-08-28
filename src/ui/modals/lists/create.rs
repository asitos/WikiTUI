use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::{centered_rect, render_modal_frame_at};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_create_new_list_modal_area(size: Rect) -> Rect {
    centered_rect(45, 25, size)
}

pub fn render_create_new_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let area = compute_create_new_list_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
        icon,
        "create new list",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );

    let lines = vec![
        Line::from(" enter name for your new list:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme::VIOLET).bold()),
            Span::styled(
                &app.lists_modal.create_input,
                Style::default().fg(theme::YELLOW).bold(),
            ),
            Span::styled("█", Style::default().fg(theme::VIOLET)),
        ]),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
