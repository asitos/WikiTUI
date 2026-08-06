use crate::app::{App, InputMode, PaneContent};
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::{Line, Span, Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top tab bar
            Constraint::Min(0),    // main workspace
            Constraint::Length(1), // bottom status bar
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
            tab_spans.push(Span::styled(
                format!(" [ {} ] ", name),
                Style::default().fg(theme::LIME).bold(),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("  {}  ", name),
                Style::default().fg(theme::DARK_GREY),
            ));
        }
    }
    let tab_bar_paragraph = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, tab_bar_area);

    // status bar
    let status_text = match app.input_mode {
        InputMode::Search => format!(
            " search: {}_ | press enter to submit, esc to cancel ",
            app.search_input
        ),
        InputMode::Normal => {
            "ctrl-s: search | j/k: navigate/scroll | enter: open article | ctrl-w s/v: split | alt-h/l: tabs | q: quit".to_string()
        }
    };
    let status_style = if app.input_mode == InputMode::Search {
        Style::default().fg(theme::BEIGE).bold()
    } else {
        Style::default().fg(theme::GREY)
    };
    let status_paragraph = Paragraph::new(status_text).style(status_style);
    f.render_widget(status_paragraph, status_area);

    // panes
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

        // determine title for pane
        let title = match &pane.content {
            PaneContent::Empty => format!(" pane {} ", pane.id),
            PaneContent::SearchResults { query, .. } => {
                format!(" pane {} - search: {} ", pane.id, query.to_lowercase())
            }
            PaneContent::ArticleText { title, .. } => {
                format!(" pane {} - {} ", pane.id, title.to_lowercase())
            }
            PaneContent::Error(_) => format!(" pane {} - error ", pane.id),
        };

        let block = Block::bordered()
            .border_style(Style::default().fg(border_color))
            .title(title);

        if pane.is_loading {
            let loading_p = Paragraph::new(" loading wikipedia data... ")
                .fg(theme::GOLD)
                .block(block);
            f.render_widget(loading_p, rect);
            continue;
        }

        match &pane.content {
            PaneContent::Empty => {
                let empty_p = Paragraph::new("press 'ctrl-s' to search wikipedia")
                    .fg(theme::GREY)
                    .block(block)
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(empty_p, rect);
            }
            PaneContent::SearchResults { items, .. } => {
                if items.is_empty() {
                    let no_res_p = Paragraph::new("no search results found")
                        .fg(theme::RED)
                        .block(block);
                    f.render_widget(no_res_p, rect);
                } else {
                    let mut lines = Vec::new();
                    for (i, item) in items.iter().enumerate() {
                        let is_selected = i == pane.selected_idx;
                        let prefix = if is_selected { "> " } else { "  " };
                        let title_style = if is_selected {
                            Style::default().fg(theme::LIME).bold()
                        } else {
                            Style::default().fg(theme::FG).bold()
                        };

                        lines.push(Line::from(vec![
                            Span::styled(prefix, title_style),
                            Span::styled(format!("{}. {}", i + 1, item.title), title_style),
                        ]));

                        if !item.snippet.is_empty() {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(&item.snippet, Style::default().fg(theme::GREY)),
                            ]));
                        }
                        lines.push(Line::from(""));
                    }
                    let results_p = Paragraph::new(lines)
                        .block(block)
                        .scroll((pane.scroll_offset as u16, 0));
                    f.render_widget(results_p, rect);
                }
            }
            PaneContent::ArticleText { text, .. } => {
                let paragraph = Paragraph::new(text.as_str())
                    .block(block)
                    .wrap(Wrap { trim: true })
                    .scroll((pane.scroll_offset as u16, 0));
                f.render_widget(paragraph, rect);
            }
            PaneContent::Error(err_msg) => {
                let err_p = Paragraph::new(format!("error: {}", err_msg))
                    .fg(theme::RED)
                    .block(block);
                f.render_widget(err_p, rect);
            }
        }
    }
}
