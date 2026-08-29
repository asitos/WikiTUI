use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_tab_titles(app: &App) -> Vec<String> {
    app.tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let loading_pane = tab.panes.iter().find(|p| p.is_loading);
            let show_icons = app.config.ui.icons;
            let (icon, raw_title, is_saved) = if let Some(pane) = loading_pane {
                let title = pane
                    .loading_title
                    .as_deref()
                    .unwrap_or("loading...")
                    .to_lowercase();
                (crate::ui::current_spinner_frame(), title, false)
            } else if let Some(active_pane) = tab.panes.get(tab.active_pane_idx) {
                match &active_pane.content {
                    PaneContent::ArticleText {
                        title, parsed_doc, ..
                    } => {
                        let saved = app.saved_lists.is_article_saved_anywhere(title);
                        let has_audio = parsed_doc.spoken_audio.is_some();
                        let icon_str = if show_icons {
                            if has_audio {
                                "󰎆"
                            } else {
                                "≡"
                            }
                        } else if has_audio {
                            "♪"
                        } else {
                            ""
                        };
                        (icon_str, title.to_lowercase(), saved)
                    }
                    PaneContent::SearchResults { query, .. } => (
                        if show_icons { "󰍉" } else { "" },
                        format!("search: {}", query.to_lowercase()),
                        false,
                    ),
                    PaneContent::Error(_) => (
                        if show_icons { "󰅚" } else { "" },
                        "error".to_string(),
                        false,
                    ),
                    PaneContent::Empty => (
                        if show_icons { "󰋜" } else { "" },
                        tab.name.to_lowercase(),
                        false,
                    ),
                }
            } else {
                (
                    if show_icons { "󰋜" } else { "" },
                    tab.name.to_lowercase(),
                    false,
                )
            };

            let star = if is_saved {
                if show_icons {
                    " ★"
                } else {
                    " *"
                }
            } else {
                ""
            };

            if icon.is_empty() {
                if app.tabs.len() > 1 {
                    format!("{} {}{}", i + 1, raw_title, star)
                } else {
                    format!("{}{}", raw_title, star)
                }
            } else if app.tabs.len() > 1 {
                format!("{} {} {}{}", icon, i + 1, raw_title, star)
            } else {
                format!("{} {}{}", icon, raw_title, star)
            }
        })
        .collect()
}

pub fn compute_visible_range(
    tab_titles: &[String],
    active_idx: usize,
    area_width: u16,
) -> (usize, usize) {
    let total_tabs = tab_titles.len();
    if total_tabs == 0 {
        return (0, 0);
    }

    let active_idx = active_idx.min(total_tabs - 1);
    let max_available_width = (area_width as usize).saturating_sub(4);

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

    (start_idx, end_idx)
}

pub fn get_tab_at_col(app: &App, area_width: u16, target_col: u16) -> Option<usize> {
    if app.tabs.is_empty() {
        return None;
    }

    let tab_titles = compute_tab_titles(app);
    let total_tabs = tab_titles.len();
    let (start_idx, end_idx) = compute_visible_range(&tab_titles, app.active_tab_idx, area_width);

    let mut col: u16 = 1;
    if start_idx > 0 {
        if target_col >= col && target_col < col + 2 {
            return Some(start_idx - 1);
        }
        col += 2;
    }

    for (i, title) in tab_titles
        .iter()
        .enumerate()
        .take(end_idx + 1)
        .skip(start_idx)
    {
        let tab_width = (title.chars().count() + 2) as u16;
        if target_col >= col && target_col < col + tab_width {
            return Some(i);
        }
        col += tab_width + 2;
    }

    if end_idx + 1 < total_tabs && target_col >= col && target_col < col + 2 {
        return Some(end_idx + 1);
    }

    None
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.tabs.is_empty() {
        return;
    }

    let tab_titles = compute_tab_titles(app);
    let total_tabs = tab_titles.len();
    let active_idx = app.active_tab_idx.min(total_tabs - 1);
    let (start_idx, end_idx) = compute_visible_range(&tab_titles, app.active_tab_idx, area.width);

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
            let active_style = Style::default().fg(theme::LIME).bg(theme::LIGHT_BG).bold();
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
