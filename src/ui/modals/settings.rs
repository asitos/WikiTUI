use super::utils::{centered_rect, render_modal_container_at};
use crate::app::{App, SettingItem};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_settings_modal_area(size: Rect) -> Rect {
    centered_rect(55, 80, size)
}

pub fn render_settings_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "󰒓" } else { "" };
    let area = compute_settings_modal_area(size);
    let inner = render_modal_container_at(
        f,
        area,
        icon,
        "settings (config.toml)",
        theme::ORANGE,
        app.config.ui.rounded_borders,
    );

    let mut lines: Vec<Line> = Vec::new();
    let mut current_section = "";

    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::styled(" [", Style::default().fg(theme::GREY)),
                Span::styled(section, Style::default().fg(theme::ORANGE).bold()),
                Span::styled("]", Style::default().fg(theme::GREY)),
            ]));
            current_section = section;
        }

        let is_focused = idx == app.settings_modal.cursor_idx;
        let prefix = if is_focused { " ▶ " } else { "   " };
        let prefix_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else {
            Style::default().fg(theme::GREY)
        };

        let label = item.label();
        let label_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else {
            Style::default().fg(theme::FG)
        };

        let value_span = match item {
            SettingItem::LikedReadonly => {
                let val = app.config.general.liked_readonly;
                bool_span(val)
            }
            SettingItem::AutoRestoreSession => {
                let val = app.config.general.auto_restore_session;
                bool_span(val)
            }
            SettingItem::ConfirmQuit => {
                let val = app.config.general.confirm_quit;
                bool_span(val)
            }
            SettingItem::RoundedBorders => {
                let val = app.config.ui.rounded_borders;
                bool_span(val)
            }
            SettingItem::Icons => {
                let val = app.config.ui.icons;
                bool_span(val)
            }
            SettingItem::ScrollIndicator => {
                let val = app.config.ui.scroll_indicator;
                bool_span(val)
            }
            SettingItem::HeadingMarker => {
                let val = app.config.reader.heading_marker;
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
            SettingItem::TocSectionNumbers => {
                let val = app.config.reader.toc_section_numbers;
                bool_span(val)
            }
            SettingItem::CodeLineNumbers => {
                let val = app.config.reader.code_line_numbers;
                bool_span(val)
            }
            SettingItem::SearchLimit => {
                let val = app.config.search.limit;
                Span::styled(
                    format!("◄  {:>2} items  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::NetworkTimeout => {
                let val = app.config.network.timeout;
                Span::styled(
                    format!("◄  {:>2}s  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::OfflineCache => {
                let val = app.config.network.offline_cache;
                bool_span(val)
            }
            SettingItem::CacheLifetime => {
                let val = app.config.network.cache_lifetime;
                Span::styled(
                    format!("◄  {:>3}h  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::MouseSupport => {
                let val = app.config.input.mouse_support;
                bool_span(val)
            }
            SettingItem::ScrollSpeed => {
                let val = app.config.input.scroll_speed;
                Span::styled(
                    format!("◄  {:>2} lines  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
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
    let focused_item = SettingItem::ALL.get(app.settings_modal.cursor_idx).copied();
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
        Span::styled("[ ON ] ", Style::default().fg(theme::LIME).bold())
    } else {
        Span::styled("[ OFF ]", Style::default().fg(theme::GREY))
    }
}

pub fn get_setting_row_at(inner: Rect, target_y: u16) -> Option<(usize, SettingItem, u16)> {
    if target_y < inner.y || target_y >= inner.y + inner.height {
        return None;
    }
    let row_in_inner = (target_y - inner.y) as usize;
    let mut cur_line = 0;
    let mut current_section = "";

    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                cur_line += 1;
            }
            cur_line += 1;
            current_section = section;
        }
        if cur_line == row_in_inner {
            let value_start_x = inner.x + 36;
            return Some((idx, *item, value_start_x));
        }
        cur_line += 1;
    }
    None
}
