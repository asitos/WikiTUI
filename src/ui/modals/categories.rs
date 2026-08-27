use super::utils::{centered_rect, render_modal_frame_at};
use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_categories_modal_area(container_rect: Rect) -> Rect {
    centered_rect(60, 55, container_rect)
}

pub fn render_categories_modal(f: &mut Frame, app: &App, size: Rect) {
    let pane = app.active_pane();
    let (title, categories) = match &pane.content {
        PaneContent::ArticleText {
            title, parsed_doc, ..
        } => (title.as_str(), &parsed_doc.categories),
        _ => return,
    };

    let total = categories.len();
    let modal_title = format!("categories · {} ({})", title.to_lowercase(), total);
    let icon = if app.config.ui.icons { "󰓹" } else { "" };

    let modal_area = compute_categories_modal_area(size);
    let modal_block = render_modal_frame_at(
        f,
        modal_area,
        icon,
        &modal_title,
        theme::TEAL,
        app.config.ui.rounded_borders,
    );

    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let selected_idx = app.categories_cursor_idx.min(total.saturating_sub(1));

    let scroll = if total <= inner_height || inner_height == 0 {
        0
    } else {
        selected_idx
            .saturating_sub(inner_height / 2)
            .min(total.saturating_sub(inner_height))
    };

    let mut lines = Vec::new();
    if categories.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  no categories found for this article.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        for (idx, cat) in categories.iter().enumerate() {
            let is_selected = idx == selected_idx;
            let prefix = if is_selected { "▶ " } else { "  " };

            let style = if is_selected {
                Style::default().fg(theme::TEAL).bold()
            } else {
                Style::default().fg(theme::FG)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    prefix,
                    if is_selected {
                        Style::default().fg(theme::TEAL).bold()
                    } else {
                        Style::default().fg(theme::GREY)
                    },
                ),
                Span::styled(cat, style),
            ]));
        }
    }

    let p = Paragraph::new(lines)
        .block(modal_block)
        .scroll((scroll as u16, 0));

    f.render_widget(p, modal_area);
}

pub fn get_category_row_at(
    area: Rect,
    row: u16,
    total_categories: usize,
    scroll: usize,
) -> Option<usize> {
    let inner_y = area.y + 1;
    let inner_h = area.height.saturating_sub(2);
    if row >= inner_y && row < inner_y + inner_h {
        let rel_row = (row - inner_y) as usize;
        let cat_idx = scroll + rel_row;
        if cat_idx < total_categories {
            return Some(cat_idx);
        }
    }
    None
}
