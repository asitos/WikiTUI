use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::{centered_rect, render_modal_frame_at};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_confirm_modal_area(size: Rect) -> Rect {
    centered_rect(50, 30, size)
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

fn build_confirm_lines(
    prompt: &str,
    detail_spans: Vec<Span<'static>>,
    action_verb: &str,
) -> Vec<Line<'static>> {
    vec![
        Line::from(prompt.to_string()),
        Line::from(""),
        Line::from(detail_spans),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y/enter] ", Style::default().fg(theme::LIME).bold()),
            Span::styled(
                format!("{}   ", action_verb),
                Style::default().fg(theme::FG),
            ),
            Span::styled("[n/esc] ", Style::default().fg(theme::GREY).bold()),
            Span::styled("cancel", Style::default().fg(theme::FG)),
        ]),
    ]
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
        Some(crate::app::ConfirmAction::DeleteList { title, .. }) => build_confirm_lines(
            "are you sure you want to delete:",
            vec![
                Span::styled("custom list: ", Style::default().fg(theme::GREY)),
                Span::styled(title.clone(), Style::default().fg(theme::YELLOW).bold()),
            ],
            "delete",
        ),
        Some(crate::app::ConfirmAction::DeleteArticle { title, .. }) => build_confirm_lines(
            "are you sure you want to delete:",
            vec![
                Span::styled("article: ", Style::default().fg(theme::GREY)),
                Span::styled(title.clone(), Style::default().fg(theme::YELLOW).bold()),
            ],
            "delete",
        ),
        Some(crate::app::ConfirmAction::ResetFeed) => build_confirm_lines(
            "are you sure you want to reset your feed?",
            vec![Span::styled(
                "all category scores and preferences will be cleared",
                Style::default().fg(theme::YELLOW),
            )],
            "reset",
        ),
        Some(crate::app::ConfirmAction::Quit) => {
            let tab_count = app.tabs.len();
            let subtext = if tab_count > 1 {
                format!("you have {} open tabs", tab_count)
            } else {
                "exit wikid reader".to_string()
            };
            build_confirm_lines(
                "are you sure you want to quit wikid?",
                vec![Span::styled(subtext, Style::default().fg(theme::YELLOW))],
                "quit",
            )
        }
        None => Vec::new(),
    };

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}
