use crate::app::App;
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::{Style, Stylize},
    widgets::{Block, Paragraph},
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical) // make rows
        .constraints([
            Constraint::Length(1), // 1 char row at the top
            Constraint::Min(0),
            Constraint::Length(1), // 1 char row at the bottom
        ])
        .split(size);

    let tab_bar_area = chunks[0];
    let main_area = chunks[1];
    let status_area = chunks[2];

    // tab bar
    let mut tab_spans = Vec::new();
    for (idx, tab) in app.tabs.iter().enumerate() {
        let name = tab.name.to_lowercase();
        if idx == app.active_tab_idx {
            tab_spans.push(format!(" [ {} ] ", name).fg(theme::LIME).bold());
        } else {
            tab_spans.push(format!("  {}  ", name).fg(theme::DARK_GREY));
        }
    }
    let tab_bar_paragraph = Paragraph::new(ratatui::text::Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, tab_bar_area);

    // status bar
    let status_text = "ctrl-t: new tab | alt-h/l: prev/next tab | ctrl-w s/v: split h/v | alt-c: close | ctrl-h/j/k/l: move | q: exit";
    let status_paragraph = Paragraph::new(status_text.dark_gray());
    f.render_widget(status_paragraph, status_area);

    let active_tab = app.active_tab();
    let rects = active_tab.layout_root.compute_rects(main_area);

    for (pane_idx, rect) in rects {
        let is_active = pane_idx == active_tab.active_pane_idx;
        let pane = &active_tab.panes[pane_idx];

        let border_color = if is_active {
            theme::PINK
        } else {
            theme::DARK_GREY
        };

        let block = Block::bordered()
            .border_style(Style::default().fg(border_color))
            .title(format!(" pane {} ", pane.id));

        let content_text = format!("pane {}", pane.id);

        let paragraph = Paragraph::new(content_text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(paragraph, rect);
    }
}
