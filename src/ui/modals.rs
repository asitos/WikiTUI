use crate::app::{App, Pane};
use crate::parser::ParsedDocument;
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph},
    Frame,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[rustfmt::skip]
pub fn render_help_modal(f: &mut Frame, size: Rect) {
    let area = centered_rect(72, 85, size);
    f.render_widget(Clear, area);

    let is_macos = cfg!(target_os = "macos");

    let mut help_text = vec![
        Line::from(vec![Span::styled(
            " navigation",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   j/k                 scroll down / up"),
        Line::from("   f/b                 scroll page down / up"),
        Line::from("   g/G                 jump to top / bottom"),
        Line::from("   ]/[                 jump to next / prev section heading"),
        Line::from("   o                   toggle table of contents"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " links & selection",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   tab/backtab         focus next / prev link"),
        Line::from("   enter               open link in current pane"),
        Line::from("   t                   open link in new tab"),
        Line::from("   s/v                 open link in horizontal / vertical split"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " panes & tabs",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
    ];

    if is_macos {
        help_text.push(Line::from("   ctrl-w s/v          split active pane horizontally / vertically"));
        help_text.push(Line::from("   ctrl-h/j/k/l        navigate focus between split panes"));
        help_text.push(Line::from("   opt-c               close active pane"));
        help_text.push(Line::from("   opt-t               create new tab"));
        help_text.push(Line::from("   opt-h/l             switch to prev / next tab"));
    } else {
        help_text.push(Line::from("   ctrl-w s/v          split active pane horizontally / vertically"));
        help_text.push(Line::from("   ctrl-h/j/k/l        navigate focus between split panes"));
        help_text.push(Line::from("   alt-c               close active pane"));
        help_text.push(Line::from("   alt-t               create new tab"));
        help_text.push(Line::from("   alt-h/l             switch to prev / next tab"));
    }

    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " search",
        Style::default().fg(theme::VIOLET).bold(),
    )]));

    help_text.push(Line::from("   ctrl-s              search wikipedia (opens new tab)"));

    help_text.push(Line::from("   i                   edit search query in current tab"));
    help_text.push(Line::from("   r                   open random wikipedia article"));
    help_text.push(Line::from("   /                   in-page text search"));
    help_text.push(Line::from("   n/N                 jump to next / prev search match"));
    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " custom lists",
        Style::default().fg(theme::VIOLET).bold(),
    )]));
    help_text.push(Line::from("   m                   save active article to custom list"));
    help_text.push(Line::from("   M                   open saved custom lists & articles viewer"));
    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " general",
        Style::default().fg(theme::VIOLET).bold(),
    )]));
    help_text.push(Line::from("   z                   toggle zen mode"));
    help_text.push(Line::from("   F                   toggle wikipedia feed mode"));
    help_text.push(Line::from("   ?                   toggle this help popup"));
    help_text.push(Line::from("   q                   quit wikid"));

    let help_block = Block::bordered()
        .border_style(Style::default().fg(theme::PINK))
        .title(Title::from(" keybindings ").alignment(Alignment::Center))
        .title(
            Title::from(Span::styled(
                " esc to close ",
                Style::default().fg(theme::GREY).italic(),
            ))
            .position(ratatui::widgets::block::Position::Bottom)
            .alignment(Alignment::Right),
        );

    let help_paragraph = Paragraph::new(help_text).block(help_block);
    f.render_widget(help_paragraph, area);
}

pub fn render_search_modal(f: &mut Frame, app: &App, size: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(size);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(28),
            Constraint::Percentage(36),
        ])
        .split(popup_layout[1])[1];

    f.render_widget(Clear, area);

    let search_block = Block::bordered()
        .border_style(Style::default().fg(theme::BEIGE))
        .title(Title::from(" search wikipedia ").alignment(Alignment::Left));

    let visible_width = (area.width as usize).saturating_sub(6);
    let chars: Vec<char> = app.search_input.chars().collect();
    let total_len = chars.len();
    let cursor_pos = app.search_cursor_pos.min(total_len);

    let mut scroll_offset = 0;
    if cursor_pos >= visible_width && visible_width > 0 {
        scroll_offset = cursor_pos + 1 - visible_width;
    }

    let end_idx = (scroll_offset + visible_width).min(total_len);
    let visible_chars = &chars[scroll_offset..end_idx];
    let rel_cursor_pos = cursor_pos.saturating_sub(scroll_offset);

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " > ",
        Style::default().fg(theme::BEIGE).bold(),
    ));

    for (i, &ch) in visible_chars.iter().enumerate() {
        if i == rel_cursor_pos {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(theme::BEIGE).fg(theme::BG).bold(),
            ));
        } else {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme::FG).bold(),
            ));
        }
    }

    if rel_cursor_pos >= visible_chars.len() {
        spans.push(Span::styled("_", Style::default().fg(theme::BEIGE).bold()));
    }

    let input_text = Line::from(spans);
    let search_paragraph = Paragraph::new(input_text).block(search_block);
    f.render_widget(search_paragraph, area);
}

pub fn render_toc_modal(
    f: &mut Frame,
    pane: &Pane,
    parsed_doc: &ParsedDocument,
    container_rect: Rect,
) {
    let toc_area = centered_rect(60, 60, container_rect);
    f.render_widget(Clear, toc_area);

    let toc_block = Block::bordered()
        .border_style(Style::default().fg(theme::LIME))
        .title(" contents ")
        .title(
            Title::from(" enter: jump | o: close ")
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Right),
        );

    let current_scroll = pane.scroll_offset;
    let active_heading_idx = parsed_doc
        .headings
        .iter()
        .rposition(|h| h.line_idx <= current_scroll)
        .unwrap_or(0);

    let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

    let mut toc_lines = Vec::new();
    for (idx, h) in parsed_doc.headings.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let indent_len = ((h.level.saturating_sub(1)) * 2) as usize;
        let indent = " ".repeat(indent_len);
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default().fg(theme::LIME).bold()
        } else {
            Style::default().fg(theme::FG)
        };

        let avail_w = (toc_area.width as usize).saturating_sub(6 + indent_len);
        let truncated_title = if h.title.chars().count() > avail_w && avail_w > 3 {
            let byte_end = h
                .title
                .char_indices()
                .nth(avail_w.saturating_sub(3))
                .map(|(i, _)| i)
                .unwrap_or(h.title.len());
            format!("{}...", &h.title[..byte_end])
        } else {
            h.title.clone()
        };

        toc_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(indent),
            Span::styled(truncated_title, style),
        ]));
    }

    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = selected_idx.saturating_sub(visible_rows / 2);

    let toc_p = Paragraph::new(toc_lines)
        .block(toc_block)
        .scroll((toc_scroll as u16, 0));
    f.render_widget(toc_p, toc_area);
}

pub fn render_category_onboarding_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(60, 80, size);
    f.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Title::from(" welcome to wikid feed! ").alignment(Alignment::Center))
        .border_style(Style::default().fg(theme::VIOLET))
        .style(Style::default().bg(theme::BG));

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

pub fn render_save_to_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(55, 60, size);
    f.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Title::from(" save article to custom list ").alignment(Alignment::Center))
        .border_style(Style::default().fg(theme::VIOLET))
        .style(Style::default().bg(theme::BG));

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" article: ", Style::default().fg(theme::GREY)),
            Span::styled(
                &app.save_modal_target_title,
                Style::default().fg(theme::YELLOW).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " select custom lists to save this article into:",
            Style::default().fg(theme::FG).italic(),
        )),
        Line::from(""),
    ];

    let list_count = app.saved_lists.lists.len();
    for (idx, list) in app.saved_lists.lists.iter().enumerate() {
        let is_focused = idx == app.save_modal_cursor_idx;
        let is_in_list = app
            .saved_lists
            .is_article_in_list(&list.id, &app.save_modal_target_title);

        let cursor_str = if is_focused { " ▶ " } else { "   " };
        let check_str = if is_in_list { "[x] " } else { "[ ] " };

        let item_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else if is_in_list {
            Style::default().fg(theme::LIME)
        } else {
            Style::default().fg(theme::FG)
        };

        lines.push(Line::from(vec![
            Span::styled(cursor_str, Style::default().fg(theme::VIOLET).bold()),
            Span::styled(
                check_str,
                if is_in_list {
                    Style::default().fg(theme::LIME).bold()
                } else {
                    Style::default().fg(theme::GREY)
                },
            ),
            Span::styled(&list.name, item_style),
            Span::styled(
                format!(" ({} articles)", list.articles.len()),
                Style::default().fg(theme::GREY),
            ),
        ]));
    }

    let is_create_focused = app.save_modal_cursor_idx == list_count;
    let create_cursor = if is_create_focused { " ▶ " } else { "   " };
    let create_style = if is_create_focused {
        Style::default().fg(theme::YELLOW).bold()
    } else {
        Style::default().fg(theme::BLUE)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(create_cursor, Style::default().fg(theme::VIOLET).bold()),
        Span::styled("[+] create new list...", create_style),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " j/k: navigate | space: toggle list | n: new list | esc: done ",
        Style::default().fg(theme::GREY).italic(),
    )]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

pub fn render_create_new_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(45, 25, size);
    f.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Title::from(" create new custom list ").alignment(Alignment::Center))
        .border_style(Style::default().fg(theme::VIOLET))
        .style(Style::default().bg(theme::BG));

    let lines = vec![
        Line::from(" enter name for your new list:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme::VIOLET).bold()),
            Span::styled(
                &app.create_list_input,
                Style::default().fg(theme::YELLOW).bold(),
            ),
            Span::styled("█", Style::default().fg(theme::VIOLET)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " enter: confirm | esc: cancel ",
            Style::default().fg(theme::GREY).italic(),
        )),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

pub fn render_saved_lists_viewer_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(80, 80, size);
    f.render_widget(Clear, area);

    let outer_block = Block::bordered()
        .title(Title::from(" saved lists & articles ").alignment(Alignment::Center))
        .border_style(Style::default().fg(theme::VIOLET))
        .style(Style::default().bg(theme::BG));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner_area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    // left lists pane
    let left_border_color = if !app.viewer_focus_right {
        theme::VIOLET
    } else {
        theme::GREY
    };
    let left_block = Block::bordered()
        .title(" custom lists ")
        .border_style(Style::default().fg(left_border_color));

    let mut list_lines = Vec::new();
    if app.saved_lists.lists.is_empty() {
        list_lines.push(Line::from(Span::styled(
            " no lists created yet",
            Style::default().fg(theme::GREY),
        )));
    } else {
        for (idx, list) in app.saved_lists.lists.iter().enumerate() {
            let is_selected = idx == app.viewer_list_idx;
            let prefix = if is_selected { " ▶ " } else { "   " };
            let style = if is_selected && !app.viewer_focus_right {
                Style::default().fg(theme::YELLOW).bold()
            } else if is_selected {
                Style::default().fg(theme::VIOLET).bold()
            } else {
                Style::default().fg(theme::FG)
            };

            list_lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme::VIOLET)),
                Span::styled(&list.name, style),
                Span::styled(
                    format!(" ({})", list.articles.len()),
                    Style::default().fg(theme::GREY),
                ),
            ]));
        }
    }

    let left_p = Paragraph::new(list_lines).block(left_block);
    f.render_widget(left_p, left_area);

    // right articles pane
    let right_border_color = if app.viewer_focus_right {
        theme::VIOLET
    } else {
        theme::GREY
    };

    let selected_list = app.saved_lists.lists.get(app.viewer_list_idx);
    let right_title = selected_list
        .map(|l| format!(" articles in '{}' ", l.name))
        .unwrap_or_else(|| " Articles ".to_string());

    let right_block = Block::bordered()
        .title(right_title)
        .border_style(Style::default().fg(right_border_color));

    let mut article_lines = Vec::new();
    if let Some(list) = selected_list {
        if list.articles.is_empty() {
            article_lines.push(Line::from(Span::styled(
                " no articles saved in this list.",
                Style::default().fg(theme::GREY).italic(),
            )));
        } else {
            for (idx, article) in list.articles.iter().enumerate() {
                let is_selected = idx == app.viewer_article_idx;
                let prefix = if is_selected { " ▶ " } else { "   " };
                let style = if is_selected && app.viewer_focus_right {
                    Style::default().fg(theme::YELLOW).bold()
                } else if is_selected {
                    Style::default().fg(theme::VIOLET).bold()
                } else {
                    Style::default().fg(theme::FG)
                };

                article_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::VIOLET)),
                    Span::styled(&article.title, style),
                ]));
            }
        }
    }

    let right_p = Paragraph::new(article_lines).block(right_block);
    f.render_widget(right_p, right_area);
}

pub fn render_confirm_delete_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(50, 30, size);
    f.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Title::from(" confirm deletion ").alignment(Alignment::Center))
        .border_style(Style::default().fg(theme::VIOLET))
        .style(Style::default().bg(theme::BG));

    let item_type = if app.pending_delete_is_list {
        "custom list"
    } else {
        "article"
    };

    let lines = vec![
        Line::from("are you sure you want to delete:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}: ", item_type), Style::default().fg(theme::GREY)),
            Span::styled(
                &app.pending_delete_title,
                Style::default().fg(theme::YELLOW).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
            Span::styled("delete   ", Style::default().fg(theme::FG)),
            Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
            Span::styled("cancel", Style::default().fg(theme::FG)),
        ]),
    ];

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}
