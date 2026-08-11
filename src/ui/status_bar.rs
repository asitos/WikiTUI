use crate::app::{App, InputMode};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match app.input_mode {
        InputMode::Search => " search wikipedia ".to_string(),
        InputMode::LocalSearch => {
            let active_pane = app.active_pane();
            let matches_info = if active_pane.local_matches.is_empty() {
                "no matches".to_string()
            } else {
                format!(
                    "match {}/{}",
                    active_pane.selected_match_idx.unwrap_or(0) + 1,
                    active_pane.local_matches.len()
                )
            };
            format!(
                " /: {}_ | {} | n: next | N: prev | esc: exit ",
                active_pane.local_search_query, matches_info
            )
        }
        InputMode::CategoryOnboarding => {
            "j/k: navigate | space: toggle | enter: start feed".to_string()
        }
        InputMode::Help => "".to_string(),
        InputMode::Normal => {
            if app.active_pane().toc_focused {
                "j/k: navigate contents | enter: jump | o: close".to_string()
            } else {
                "ctrl-s: search | r: random | ?: help | q: quit".to_string()
            }
        }
    };

    let status_style = match app.input_mode {
        InputMode::Search => Style::default().fg(theme::BEIGE).bold(),
        InputMode::LocalSearch => Style::default().fg(theme::YELLOW).bold(),
        InputMode::CategoryOnboarding => Style::default().fg(theme::VIOLET).bold(),
        InputMode::Help => Style::default().fg(theme::GREY).bold(),
        InputMode::Normal => Style::default().fg(theme::GREY),
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Center);
    f.render_widget(status_paragraph, area);
}
