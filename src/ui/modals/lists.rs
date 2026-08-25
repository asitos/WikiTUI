use super::utils::{
    centered_rect, create_checkbox_line, create_selectable_line, render_modal_frame_at,
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

pub fn compute_save_to_list_modal_area(size: Rect) -> Rect {
    centered_rect(55, 60, size)
}

pub fn compute_create_new_list_modal_area(size: Rect) -> Rect {
    centered_rect(45, 25, size)
}

pub fn compute_confirm_modal_area(size: Rect) -> Rect {
    centered_rect(50, 30, size)
}

pub fn render_save_to_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let area = compute_save_to_list_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
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
    let area = compute_create_new_list_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
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

pub fn compute_saved_lists_viewer_areas(size: Rect) -> (Rect, Rect, Rect) {
    let container_area = centered_rect(80, 80, size);
    let inner_area = Rect::new(
        container_area.x + 1,
        container_area.y + 1,
        container_area.width.saturating_sub(2),
        container_area.height.saturating_sub(2),
    );
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner_area);
    (container_area, chunks[0], chunks[1])
}

pub fn render_saved_lists_viewer_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "★" } else { "" };
    let (container_area, left_area, right_area) = compute_saved_lists_viewer_areas(size);
    f.render_widget(ratatui::widgets::Clear, container_area);
    let block = super::utils::create_modal_block(
        icon,
        "saved lists & articles",
        theme::VIOLET,
        app.config.ui.rounded_borders,
    );
    f.render_widget(block, container_area);

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

    let left_visible_rows = (left_area.height.saturating_sub(2)) as usize;
    let left_scroll = compute_list_viewer_scroll(
        app.lists_modal.viewer_list_idx,
        left_visible_rows,
        app.saved_lists.lists.len(),
    );
    let left_p = Paragraph::new(list_lines)
        .block(left_block)
        .scroll((left_scroll as u16, 0));
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
    let right_total = selected_list.map(|l| l.articles.len()).unwrap_or(0);
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

    let right_visible_rows = (right_area.height.saturating_sub(2)) as usize;
    let right_scroll = compute_list_viewer_scroll(
        app.lists_modal.viewer_article_idx,
        right_visible_rows,
        right_total,
    );
    let right_p = Paragraph::new(article_lines)
        .block(right_block)
        .scroll((right_scroll as u16, 0));
    f.render_widget(right_p, right_area);
}

pub fn compute_list_viewer_scroll(
    cursor_idx: usize,
    visible_rows: usize,
    total_items: usize,
) -> usize {
    if total_items <= visible_rows || visible_rows == 0 {
        0
    } else {
        cursor_idx
            .saturating_sub(visible_rows / 2)
            .min(total_items.saturating_sub(visible_rows))
    }
}

pub fn get_saved_lists_viewer_item_at(
    app: &App,
    is_right: bool,
    area: Rect,
    target_y: u16,
) -> Option<usize> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let row_offset = (target_y - (area.y + 1)) as usize;
    let visible_rows = (area.height.saturating_sub(2)) as usize;
    if is_right {
        let selected_list = app.saved_lists.lists.get(app.lists_modal.viewer_list_idx)?;
        let total = selected_list.articles.len();
        let scroll =
            compute_list_viewer_scroll(app.lists_modal.viewer_article_idx, visible_rows, total);
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    } else {
        let total = app.saved_lists.lists.len();
        let scroll =
            compute_list_viewer_scroll(app.lists_modal.viewer_list_idx, visible_rows, total);
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveToListHit {
    Toggle(usize),
    CreateNew,
}

pub fn get_save_to_list_item_at(app: &App, area: Rect, target_y: u16) -> Option<SaveToListHit> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let inner_y = area.y + 1;
    let custom_lists_count = app
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .count();
    let start_row = inner_y + 4;
    if target_y >= start_row && target_y < start_row + (custom_lists_count as u16) {
        Some(SaveToListHit::Toggle((target_y - start_row) as usize))
    } else if target_y == start_row + (custom_lists_count as u16) + 1 {
        Some(SaveToListHit::CreateNew)
    } else {
        None
    }
}

pub fn get_confirm_button_at(app: &App, area: Rect, col: u16, row: u16) -> Option<char> {
    let btn_row = area.y + 5;
    if row != btn_row {
        return None;
    }
    let action_str = match &app.confirm_action {
        Some(crate::app::ConfirmAction::DeleteList { .. }) => "delete",
        Some(crate::app::ConfirmAction::DeleteArticle { .. }) => "delete",
        Some(crate::app::ConfirmAction::ResetFeed) => "delete",
        Some(crate::app::ConfirmAction::Quit) => "quit",
        None => return None,
    };
    let yes_str = format!("[y/enter] {}", action_str);
    let no_str = "[n/esc] cancel";
    let gap = "   ";
    let total_len = yes_str.len() + gap.len() + no_str.len();
    let inner_width = (area.width.saturating_sub(2)) as usize;
    let start_x = area.x + 1 + (inner_width.saturating_sub(total_len) / 2) as u16;
    let yes_end_x = start_x + yes_str.len() as u16;
    let no_start_x = yes_end_x + gap.len() as u16;
    let no_end_x = no_start_x + no_str.len() as u16;

    if col >= start_x && col < yes_end_x {
        Some('y')
    } else if col >= no_start_x && col < no_end_x {
        Some('n')
    } else {
        None
    }
}

pub fn render_confirm_modal(f: &mut Frame, app: &App, size: Rect) {
    let modal_title = match &app.confirm_action {
        Some(crate::app::ConfirmAction::ResetFeed) => "confirm feed reset",
        Some(crate::app::ConfirmAction::Quit) => "confirm quit",
        _ => "confirm deletion",
    };

    let icon = if app.config.ui.icons { "󰅚" } else { "" };
    let area = compute_confirm_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
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
                Line::from(Span::styled(subtext, Style::default().fg(theme::YELLOW))),
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
