use super::utils::centered_rect;
use crate::app::App;
use crate::session::SessionState;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn render_restore_session_modal(
    f: &mut Frame,
    _app: &App,
    state: &SessionState,
    size: Rect,
) {
    let area = centered_rect(55, 25, size);
    f.render_widget(Clear, area);

    let block = Block::bordered()
        .title(" restore session ")
        .border_style(Style::default().fg(theme::VIOLET));

    let tab_count = state.tabs.len();
    let total_articles: usize = state
        .tabs
        .iter()
        .map(|t| t.panes.iter().filter(|p| p.title.is_some()).count())
        .sum();

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(" found previous session with "),
            Span::styled(
                format!("{} tab(s)", tab_count),
                Style::default().fg(theme::YELLOW).bold(),
            ),
            Span::raw(" & "),
            Span::styled(
                format!("{} article(s)", total_articles),
                Style::default().fg(theme::TEAL).bold(),
            ),
            Span::raw("."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" would you like to restore your session?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y / enter] ", Style::default().fg(theme::LIME).bold()),
            Span::raw("restore session   "),
            Span::styled(" [n / esc] ", Style::default().fg(theme::RED).bold()),
            Span::raw("start fresh"),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
