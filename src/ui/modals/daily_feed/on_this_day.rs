use super::html::{parse_onthisday_event, wrap_story_spans};
use super::types::{OnThisDayTab, SpanStyle};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

const MONTH_NAMES: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

pub fn today_date_str() -> String {
    let (_y, m, d) = crate::api::daily_feed::utc_today();
    let month_str = MONTH_NAMES
        .get(m.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    format!("{} {}", month_str, d)
}

pub fn render_on_this_day_modal(
    f: &mut Frame,
    app: &App,
    modal_area: Rect,
    modal_block: Block,
    selected_idx: usize,
    link_idx: usize,
    otd_tab: OnThisDayTab,
) {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return,
    };

    let archive = feed.onthisday_all.as_ref();
    let events_slice: &[crate::api::daily_feed::OnThisDayEvent] = if let Some(arch) = archive {
        match otd_tab {
            OnThisDayTab::Events => {
                if !arch.events.is_empty() {
                    &arch.events
                } else {
                    &feed.onthisday
                }
            }
            OnThisDayTab::Births => &arch.births,
            OnThisDayTab::Deaths => &arch.deaths,
            OnThisDayTab::Holidays => &arch.holidays,
        }
    } else {
        &feed.onthisday
    };

    let selected_event = events_slice.get(selected_idx);
    let focused_page = selected_event.and_then(|ev| {
        let (_, event_links) = parse_onthisday_event(&ev.text, &ev.pages);
        let active_link_idx = link_idx.min(event_links.len().saturating_sub(1));
        let target = event_links.get(active_link_idx)?;
        ev.pages
            .iter()
            .find(|p| &p.title == target || p.display_title() == *target)
            .or_else(|| ev.pages.first())
    });

    let mut modal_block = modal_block;
    if let Some(page) = focused_page {
        if let Some(desc) = page.description.as_deref().filter(|d| !d.is_empty()) {
            let icon = if app.config.ui.icons { "󰋼 " } else { "" };
            let max_desc_len =
                (modal_area.width as usize).saturating_sub(page.display_title().len() + 10);
            let clean_desc = if desc.len() > max_desc_len && max_desc_len > 3 {
                let byte_end = desc
                    .char_indices()
                    .nth(max_desc_len.saturating_sub(3))
                    .map(|(i, _)| i)
                    .unwrap_or(desc.len());
                format!("{}...", &desc[..byte_end])
            } else {
                desc.to_string()
            };
            let footer_line = Line::from(vec![
                Span::styled(
                    format!(" {}{}: ", icon, page.display_title()),
                    Style::default().fg(theme::BLUE).bold(),
                ),
                Span::styled(
                    format!("{} ", clean_desc),
                    Style::default().fg(theme::GREY).italic(),
                ),
            ]);
            modal_block = modal_block.title_bottom(footer_line);
        }
    }

    let mut lines = Vec::new();
    let avail_w = (modal_area.width as usize).saturating_sub(4);

    let (ev_count, b_count, d_count, h_count) = if let Some(arch) = archive {
        (
            if !arch.events.is_empty() {
                arch.events.len()
            } else {
                feed.onthisday.len()
            },
            arch.births.len(),
            arch.deaths.len(),
            arch.holidays.len(),
        )
    } else {
        (feed.onthisday.len(), 0, 0, 0)
    };

    let tabs = [
        (OnThisDayTab::Events, "1", "Events", ev_count),
        (OnThisDayTab::Births, "2", "Births", b_count),
        (OnThisDayTab::Deaths, "3", "Deaths", d_count),
        (OnThisDayTab::Holidays, "4", "Holidays", h_count),
    ];

    let mut tab_spans = vec![Span::raw("  ")];
    for (i, (t, num, label, count)) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled("   ", Style::default().fg(theme::DARK_GREY)));
        }
        let is_active = otd_tab == *t;
        if is_active {
            tab_spans.push(Span::styled(
                format!("[{}] {} ({})", num, label, count),
                Style::default()
                    .fg(theme::BLUE)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("[{}] ", num),
                Style::default().fg(theme::DARK_GREY),
            ));
            tab_spans.push(Span::styled(
                format!("{} ({})", label, count),
                Style::default().fg(theme::GREY),
            ));
        }
    }
    lines.push(Line::from(tab_spans));
    let div_w = avail_w.saturating_sub(2);
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─".repeat(div_w), Style::default().fg(theme::DARK_GREY)),
    ]));
    lines.push(Line::from(""));

    let mut line_offsets = Vec::new();
    if events_slice.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  no entries available in this category.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        for (idx, event) in events_slice.iter().enumerate() {
            line_offsets.push(lines.len());
            let is_selected = idx == selected_idx;
            let current_year = crate::api::daily_feed::utc_today().0 as i32;
            let (year_str, elapsed_str) = match event.year {
                Some(y) if y < 0 => {
                    let yrs = current_year + y.abs();
                    (format!("{} BC", y.abs()), format!("({} yrs ago) ", yrs))
                }
                Some(y) => {
                    let yrs = current_year - y;
                    if yrs == 0 {
                        (format!("{}", y), "(this year) ".to_string())
                    } else {
                        (format!("{}", y), format!("({} yrs ago) ", yrs))
                    }
                }
                None => ("Holiday".to_string(), String::new()),
            };
            let (chunks, event_links) = parse_onthisday_event(&event.text, &event.pages);
            let active_link_idx = if is_selected {
                link_idx.min(event_links.len().saturating_sub(1))
            } else {
                0
            };
            let badge_prefix = format!("[ {} ] {}", year_str, elapsed_str);
            let badge_len = badge_prefix.chars().count();
            let text_w = avail_w.saturating_sub(badge_len + 3);

            let wrapped_lines = wrap_story_spans(&chunks, text_w + 3);
            for (line_idx, line_words) in wrapped_lines.into_iter().enumerate() {
                let mut spans = Vec::new();
                if line_idx == 0 {
                    let prefix = if is_selected { " ▶ " } else { "   " };
                    let prefix_style = if is_selected {
                        Style::default().fg(theme::BLUE).bold()
                    } else {
                        Style::default().fg(theme::GREY)
                    };
                    spans.push(Span::styled(prefix, prefix_style));
                    spans.push(Span::styled("[ ", Style::default().fg(theme::DARK_GREY)));
                    spans.push(Span::styled(
                        year_str.clone(),
                        Style::default().fg(theme::BLUE).bold(),
                    ));
                    spans.push(Span::styled(" ] ", Style::default().fg(theme::DARK_GREY)));
                    if !elapsed_str.is_empty() {
                        spans.push(Span::styled(
                            elapsed_str.clone(),
                            Style::default().fg(theme::GREY).italic(),
                        ));
                    }
                } else {
                    let pad_len = 3 + badge_len;
                    spans.push(Span::raw(" ".repeat(pad_len)));
                }

                for (text, style) in line_words {
                    let span_style = match style {
                        SpanStyle::Normal => {
                            if is_selected {
                                Style::default().fg(theme::FG).bold()
                            } else {
                                Style::default().fg(theme::FG)
                            }
                        }
                        SpanStyle::Bold => Style::default().fg(theme::FG).bold(),
                        SpanStyle::Italic => Style::default().fg(theme::GREY).italic(),
                        SpanStyle::Link {
                            link_idx: l_idx, ..
                        } => {
                            if is_selected && l_idx == active_link_idx {
                                Style::default()
                                    .fg(theme::VIOLET)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else if app.config.reader.underline_links {
                                Style::default()
                                    .fg(theme::BLUE)
                                    .add_modifier(Modifier::UNDERLINED)
                            } else {
                                Style::default().fg(theme::BLUE)
                            }
                        }
                        SpanStyle::BoldLink {
                            link_idx: l_idx, ..
                        } => {
                            if is_selected && l_idx == active_link_idx {
                                Style::default()
                                    .fg(theme::VIOLET)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else if app.config.reader.underline_links {
                                Style::default()
                                    .fg(theme::BLUE)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else {
                                Style::default().fg(theme::BLUE).bold()
                            }
                        }
                    };
                    spans.push(Span::styled(text, span_style));
                }
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
        }
    }

    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();
    let scroll = if total_lines <= inner_height || inner_height == 0 {
        0
    } else {
        let target_line = line_offsets.get(selected_idx).copied().unwrap_or(0);
        target_line
            .saturating_sub(inner_height / 3)
            .min(total_lines.saturating_sub(inner_height))
    };

    let p = Paragraph::new(lines)
        .block(modal_block)
        .scroll((scroll as u16, 0));
    f.render_widget(p, modal_area);
}
