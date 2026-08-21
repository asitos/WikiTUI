use super::utils::{create_checkbox_line, render_modal_frame};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_category_onboarding_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "󰠱" } else { "" };
    let (area, block) = render_modal_frame(
        f,
        size,
        60,
        80,
        icon,
        "welcome to wikid feed",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );

    let mut lines = vec![
        Line::from(Span::styled(
            " pick some categories to get started (optional)",
            Style::default().fg(theme::FG).italic(),
        )),
        Line::from(""),
    ];

    for (idx, (display_name, _, _)) in crate::feed::profile::POPULAR_CATEGORIES.iter().enumerate() {
        let is_focused = idx == app.onboarding_cursor_idx;
        let is_checked = app.onboarding_selected.get(idx).copied().unwrap_or(false);

        lines.push(create_checkbox_line(
            display_name,
            is_focused,
            is_checked,
            None,
            theme::VIOLET,
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " j/k: navigate | space: toggle | enter: start feed",
        Style::default().fg(theme::GREY).italic(),
    )]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
