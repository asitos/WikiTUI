use super::utils::{centered_rect, create_modal_block};
use crate::app::{App, SettingItem};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

pub fn render_settings_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(64, 50, size);
    f.render_widget(Clear, area);

    let modal_block =
        create_modal_block("󰒓", "settings (config.toml)", theme::ORANGE);

    let inner = modal_block.inner(area);
    f.render_widget(modal_block, area);

    let mut lines: Vec<Line> = Vec::new();
    let mut current_section = "";

    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    format!("[{}]", section),
                    Style::default()
                        .fg(theme::VIOLET)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            current_section = section;
        }

        let is_selected = idx == app.settings_cursor_idx;
        let prefix = if is_selected { " ▸ " } else { "   " };
        let prefix_style = if is_selected {
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DARK_GREY)
        };

        let label_style = if is_selected {
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::FG)
        };

        let label = item.label();
        let value_span = match item {
            SettingItem::LikedReadonly => {
                let val = app.config.general.liked_readonly;
                bool_span(val)
            }
            SettingItem::AutoRestoreSession => {
                let val = app.config.general.auto_restore_session;
                bool_span(val)
            }
            SettingItem::ScrollLines => {
                let val = app.config.reader.scroll_lines;
                Span::styled(
                    format!("◄  {:>2} lines  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::UnderlineLinks => {
                let val = app.config.reader.underline_links;
                bool_span(val)
            }
            SettingItem::ShowFootnotes => {
                let val = app.config.reader.show_footnotes;
                bool_span(val)
            }
            SettingItem::ShowExternalLinks => {
                let val = app.config.reader.show_external_links;
                bool_span(val)
            }
        };

        let pad_len = 34_usize.saturating_sub(label.len());
        let dots = ".".repeat(pad_len.max(2));

        lines.push(Line::from(vec![
            Span::styled(prefix, prefix_style),
            Span::styled(label, label_style),
            Span::styled(format!(" {} ", dots), Style::default().fg(theme::DARK_GREY)),
            value_span,
        ]));
    }

    lines.push(Line::from(""));
    let focused_item = SettingItem::ALL.get(app.settings_cursor_idx).copied();
    if let Some(item) = focused_item {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                item.description(),
                Style::default().fg(theme::BEIGE).italic(),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            "space/enter: toggle  │  h/l: adjust  │  r: reset",
            Style::default().fg(theme::GREY),
        ),
    ]));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn bool_span(val: bool) -> Span<'static> {
    if val {
        Span::styled("[✔ ON] ", Style::default().fg(theme::LIME).bold())
    } else {
        Span::styled("[✖ OFF]", Style::default().fg(theme::GREY))
    }
}
