use crate::app::{App, InputMode};
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::Paragraph,
    Frame,
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
        InputMode::SaveToList => {
            "j/k: navigate | space: toggle | c: new list | esc: done".to_string()
        }
        InputMode::CreateNewList => "enter: confirm | esc: cancel".to_string(),
        InputMode::SavedListsViewer => {
            "h/l: switch pane | j/k: navigate | enter: open | d: delete".to_string()
        }
        InputMode::ConfirmDelete => "".to_string(),
        InputMode::Normal => {
            let pane = app.active_pane();
            if pane.toc_focused {
                "j/k: navigate contents | enter: jump | o: close".to_string()
            } else if let Some(link) = pane.focused_link().filter(|l| l.is_external()) {
                format!("↗ external link: {} | enter/y: copy URL", link.title)
            } else {
                "ctrl-s: search | r: random | F: feed | ?: help | q: quit".to_string()
            }
        }
    };

    let status_style = match app.input_mode {
        InputMode::Search => Style::default().fg(theme::BEIGE).bold(),
        InputMode::LocalSearch => Style::default().fg(theme::YELLOW).bold(),
        InputMode::CategoryOnboarding => Style::default().fg(theme::VIOLET).bold(),
        InputMode::SaveToList
        | InputMode::CreateNewList
        | InputMode::SavedListsViewer
        | InputMode::ConfirmDelete => Style::default().fg(theme::VIOLET).bold(),
        InputMode::Help => Style::default().fg(theme::GREY).bold(),
        InputMode::Normal => {
            if app
                .active_pane()
                .focused_link()
                .is_some_and(|l| l.is_external())
            {
                Style::default().fg(theme::TEAL).bold()
            } else {
                Style::default().fg(theme::GREY)
            }
        }
    };

    let (status_text, status_style) = if let Some((ref msg, time)) = app.status_message {
        if time.elapsed().as_secs_f32() < 3.0 {
            (
                format!(" {} ", msg),
                Style::default().fg(theme::LIME).bold(),
            )
        } else {
            (status_text, status_style)
        }
    } else {
        (status_text, status_style)
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Center);
    f.render_widget(status_paragraph, area);
}
