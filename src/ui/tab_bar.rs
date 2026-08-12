use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let mut tab_spans = Vec::new();
    tab_spans.push(Span::styled(" [ ", Style::default().fg(theme::GREY)));
    for (i, tab) in app.tabs.iter().enumerate() {
        let is_active = i == app.active_tab_idx;
        let style = if is_active {
            Style::default().fg(theme::LIME).bold()
        } else {
            Style::default().fg(theme::GREY)
        };

        let tab_title = if let Some(active_pane) = tab.panes.get(tab.active_pane_idx) {
            match &active_pane.content {
                PaneContent::ArticleText { title, .. } => title.to_lowercase(),
                PaneContent::SearchResults { query, .. } => {
                    format!("search: {}", query.to_lowercase())
                }
                PaneContent::Error(_) => "error".to_string(),
                PaneContent::Empty => tab.name.to_lowercase(),
            }
        } else {
            tab.name.to_lowercase()
        };

        tab_spans.push(Span::styled(tab_title, style));
        if i < app.tabs.len() - 1 {
            tab_spans.push(Span::styled(" | ", Style::default().fg(theme::GREY)));
        }
    }
    tab_spans.push(Span::styled(" ] ", Style::default().fg(theme::GREY)));
    let tab_bar_paragraph = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, area);
}
