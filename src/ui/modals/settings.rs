use super::utils::render_modal_container;
use crate::app::{App, SettingItem};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_settings_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.config.ui.icons { "󰒓" } else { "" };
    let inner = render_modal_container(
        f,
        size,
        50,
        75,
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

        let is_focused = idx == app.settings_cursor_idx;
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
