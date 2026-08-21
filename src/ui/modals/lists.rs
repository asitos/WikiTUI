use super::utils::{
    create_checkbox_line, create_selectable_line, render_modal_container, render_modal_frame,
};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render_save_to_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let (area, block) = render_modal_frame(
        f,
        size,
        55,
        60,
        icon,
        "save to list",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" article: ", Style::default().fg(theme::GREY)),
            Span::styled(
                &app.lists_modal.target_title,
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

    let custom_lists: Vec<_> = app
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .collect();
    let list_count = custom_lists.len();
    for (idx, list) in custom_lists.iter().enumerate() {
        let is_focused = idx == app.lists_modal.save_cursor_idx;
        let is_in_list = app
            .saved_lists
            .is_article_in_list(&list.id, &app.lists_modal.target_title);
        let suffix = format!(" ({} articles)", list.articles.len());

        lines.push(create_checkbox_line(
            &list.name,
            is_focused,
            is_in_list,
            Some(&suffix),
            theme::VIOLET,
        ));
    }

    let is_create_focused = app.lists_modal.save_cursor_idx == list_count;
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

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

pub fn render_create_new_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let (area, block) = render_modal_frame(
        f,
        size,
        45,
        25,
        icon,
        "create new list",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );

    let lines = vec![
        Line::from(" enter name for your new list:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme::VIOLET).bold()),
            Span::styled(
                &app.lists_modal.create_input,
                Style::default().fg(theme::YELLOW).bold(),
            ),
            Span::styled("█", Style::default().fg(theme::VIOLET)),
        ]),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

pub fn render_saved_lists_viewer_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let inner_area = render_modal_container(
        f,
        size,
        80,
        80,
        icon,
        "saved lists & articles",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner_area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    let left_border_color = if !app.lists_modal.viewer_focus_right {
        theme::VIOLET
    } else {
        theme::GREY
    };
    let border_type = if app.config.ui.rounded_borders {
        ratatui::widgets::BorderType::Rounded
    } else {
        ratatui::widgets::BorderType::Plain
    };
    let left_block = Block::bordered()
        .border_type(border_type)
        .title(" custom lists ")
        .border_style(Style::default().fg(left_border_color));

    let mut list_lines = Vec::new();
    if app.saved_lists.lists.is_empty() {
        list_lines.push(Line::from(Span::styled(
            " no lists created yet.",
            Style::default().fg(theme::GREY).italic(),
        )));
    } else {
        for (idx, list) in app.saved_lists.lists.iter().enumerate() {
            let is_selected = idx == app.lists_modal.viewer_list_idx;
            let is_active = !app.lists_modal.viewer_focus_right;
            let suffix = format!(" ({})", list.articles.len());

            list_lines.push(create_selectable_line(
                &list.name,
                is_selected,
                is_active,
                theme::VIOLET,
                Some(&suffix),
            ));
        }
    }

    let left_p = Paragraph::new(list_lines).block(left_block);
    f.render_widget(left_p, left_area);

    let right_border_color = if app.lists_modal.viewer_focus_right {
        theme::YELLOW
    } else {
        theme::GREY
    };

    let selected_list = app.saved_lists.lists.get(app.lists_modal.viewer_list_idx);
    let right_title = selected_list
        .map(|l| format!(" articles in '{}' ", l.name))
        .unwrap_or_else(|| " Articles ".to_string());

    let right_block = Block::bordered()
        .border_type(border_type)
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
                let is_selected = idx == app.lists_modal.viewer_article_idx;
                let is_active = app.lists_modal.viewer_focus_right;

                article_lines.push(create_selectable_line(
                    article,
                    is_selected,
                    is_active,
                    theme::VIOLET,
                    None,
                ));
            }
        }
    }

    let right_p = Paragraph::new(article_lines).block(right_block);
    f.render_widget(right_p, right_area);
}

pub fn render_confirm_modal(f: &mut Frame, app: &App, size: Rect) {
    let modal_title = match &app.confirm_action {
        Some(crate::app::ConfirmAction::ResetFeed) => "confirm feed reset",
        Some(crate::app::ConfirmAction::Quit) => "confirm quit",
        _ => "confirm deletion",
    };

    let icon = if app.config.ui.icons { "󰅚" } else { "" };
    let (area, block) = render_modal_frame(
        f,
        size,
        50,
        30,
        icon,
        modal_title,
        theme::RED,
        app.config.ui.rounded_borders,
    );

    let lines = match &app.confirm_action {
        Some(crate::app::ConfirmAction::DeleteList { title, .. }) => {
            vec![
                Line::from("are you sure you want to delete:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("custom list: ", Style::default().fg(theme::GREY)),
                    Span::styled(title, Style::default().fg(theme::YELLOW).bold()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
                    Span::styled("delete   ", Style::default().fg(theme::FG)),
                    Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
                    Span::styled("cancel", Style::default().fg(theme::FG)),
                ]),
            ]
        }
        Some(crate::app::ConfirmAction::DeleteArticle { title, .. }) => {
            vec![
                Line::from("are you sure you want to delete:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("article: ", Style::default().fg(theme::GREY)),
                    Span::styled(title, Style::default().fg(theme::YELLOW).bold()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
                    Span::styled("delete   ", Style::default().fg(theme::FG)),
                    Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
                    Span::styled("cancel", Style::default().fg(theme::FG)),
                ]),
            ]
        }
        Some(crate::app::ConfirmAction::ResetFeed) => {
            vec![
                Line::from("are you sure you want to reset your feed?"),
                Line::from(""),
                Line::from(Span::styled(
                    "all category scores and preferences will be cleared",
                    Style::default().fg(theme::YELLOW),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
                    Span::styled("reset   ", Style::default().fg(theme::FG)),
                    Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
                    Span::styled("cancel", Style::default().fg(theme::FG)),
                ]),
            ]
        }
        Some(crate::app::ConfirmAction::Quit) => {
            let tab_count = app.tabs.len();
            let subtext = if tab_count > 1 {
                format!("you have {} open tabs", tab_count)
            } else {
                "exit wikid reader".to_string()
            };
            vec![
                Line::from("are you sure you want to quit wikid?"),
                Line::from(""),
                Line::from(Span::styled(
                    subtext,
                    Style::default().fg(theme::YELLOW),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
                    Span::styled("quit   ", Style::default().fg(theme::FG)),
                    Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
                    Span::styled("cancel", Style::default().fg(theme::FG)),
                ]),
            ]
        }
        None => Vec::new(),
    };

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}
