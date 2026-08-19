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
    if app.tabs.is_empty() {
        return;
    }

    let tab_titles: Vec<String> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let is_loading = tab.panes.iter().any(|p| p.is_loading);
            let (icon, raw_title) = if is_loading {
                (
                    crate::ui::current_spinner_frame().to_string(),
                    "loading...".to_string(),
                )
            } else if let Some(active_pane) = tab.panes.get(tab.active_pane_idx) {
                match &active_pane.content {
                    PaneContent::ArticleText { title, .. } => {
                        ("≡".to_string(), title.to_lowercase())
                    }
                    PaneContent::SearchResults { query, .. } => {
                        ("󰍉".to_string(), format!("search: {}", query.to_lowercase()))
                    }
                    PaneContent::Error(_) => ("󰅚".to_string(), "error".to_string()),
                    PaneContent::Empty => ("󰋜".to_string(), tab.name.to_lowercase()),
                }
            } else {
                ("󰋜".to_string(), tab.name.to_lowercase())
            };

            if app.tabs.len() > 1 {
                format!("{} {} {}", icon, i + 1, raw_title)
            } else {
                format!("{} {}", icon, raw_title)
            }
        })
        .collect();

    let total_tabs = app.tabs.len();
    let active_idx = app.active_tab_idx.min(total_tabs - 1);

    let max_available_width = (area.width as usize).saturating_sub(4);

    let mut start_idx = active_idx;
    let mut end_idx = active_idx;
    let mut current_width = tab_titles[active_idx].chars().count() + 4;

    loop {
        let mut expanded = false;

        if end_idx + 1 < total_tabs {
            let next_w = tab_titles[end_idx + 1].chars().count() + 4;
            if current_width + next_w <= max_available_width {
                end_idx += 1;
                current_width += next_w;
                expanded = true;
            }
        }

        if start_idx > 0 {
            let prev_w = tab_titles[start_idx - 1].chars().count() + 4;
            if current_width + prev_w <= max_available_width {
                start_idx -= 1;
                current_width += prev_w;
                expanded = true;
            }
        }

        if !expanded {
            break;
        }
    }

    let mut tab_spans = Vec::new();
    tab_spans.push(Span::raw(" "));

    if start_idx > 0 {
        tab_spans.push(Span::styled(
            "< ",
            Style::default().fg(theme::YELLOW).bold(),
        ));
    }

    for (i, title) in tab_titles
        .iter()
        .enumerate()
        .take(end_idx + 1)
        .skip(start_idx)
    {
        let is_active = i == active_idx;
        if is_active {
            let active_style = Style::default()
                .fg(theme::LIME)
                .bg(theme::LIGHT_BG)
                .bold();
            tab_spans.push(Span::styled(format!(" {} ", title), active_style));
        } else {
            let inactive_style = Style::default().fg(theme::GREY);
            tab_spans.push(Span::styled(format!(" {} ", title), inactive_style));
        }
        tab_spans.push(Span::raw("  "));
    }

    if end_idx + 1 < total_tabs {
        tab_spans.push(Span::styled(
            "> ",
            Style::default().fg(theme::YELLOW).bold(),
        ));
    }

    let tab_bar_paragraph = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar_paragraph, area);
}
