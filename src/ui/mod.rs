pub mod modals;
pub mod pane_view;
pub mod status_bar;
pub mod tab_bar;

use crate::app::{App, InputMode};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::Paragraph,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    if app.feed.active {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        let feed_area = chunks[0];
        let status_area = chunks[1];

        crate::feed::ui::render_feed_view(f, &app.feed, feed_area);

        let status_p = Paragraph::new(" j/k: navigate | l: like | enter: read | esc: exit ")
            .style(Style::default().fg(theme::GREY))
            .alignment(Alignment::Center);
        f.render_widget(status_p, status_area);
    } else if app.zen_mode {
        let zen_area = modals::centered_rect(80, 90, size);
        pane_view::render_single_active_pane(f, app, zen_area);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        let tab_bar_area = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        tab_bar::render(f, app, tab_bar_area);
        status_bar::render(f, app, status_area);
        pane_view::render_panes(f, app, main_area);
    }

    if app.input_mode == InputMode::Help {
        modals::render_help_modal(f, size);
    }

    if app.input_mode == InputMode::Search {
        modals::render_search_modal(f, app, size);
    }

    if app.input_mode == InputMode::CategoryOnboarding {
        modals::render_category_onboarding_modal(f, app, size);
    }
}
