use super::utils::{centered_rect, create_modal_block};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

pub fn render_category_onboarding_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(60, 80, size);
    f.render_widget(Clear, area);

    let icon = if app.config.ui.icons { "󰠱" } else { "" };
    let block = create_modal_block(
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

        let cursor_str = if is_focused { " ▶ " } else { "   " };
        let check_str = if is_checked { "[x] " } else { "[ ] " };

        let item_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else if is_checked {
            Style::default().fg(theme::LIME)
        } else {
            Style::default().fg(theme::FG)
        };

        lines.push(Line::from(vec![
            Span::styled(cursor_str, Style::default().fg(theme::VIOLET).bold()),
            Span::styled(
                check_str,
                if is_checked {
                    Style::default().fg(theme::LIME).bold()
                } else {
                    Style::default().fg(theme::GREY)
                },
            ),
            Span::styled(*display_name, item_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " j/k: navigate | space: toggle | enter: start feed",
        Style::default().fg(theme::GREY).italic(),
    )]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
