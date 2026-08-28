use super::utils::{centered_rect, create_selectable_line, render_modal_frame_at};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DailyFeedKind {
    News,
    OnThisDay,
    MostRead,
}

#[derive(Clone, Debug)]
pub struct DailyFeedModalState {
    pub kind: DailyFeedKind,
    pub cursor_idx: usize,
}

impl Default for DailyFeedModalState {
    fn default() -> Self {
        Self {
            kind: DailyFeedKind::News,
            cursor_idx: 0,
        }
    }
}

pub struct FeedEntry {
    pub title: String,
    pub target_article: String,
    pub suffix: Option<String>,
}

pub fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            if chars.clone().take(3).collect::<String>() == "!--" {
                chars.next();
                chars.next();
                chars.next();
                let mut dashes = 0;
                for nc in chars.by_ref() {
                    if nc == '-' {
                        dashes += 1;
                    } else if nc == '>' && dashes >= 2 {
                        break;
                    } else {
                        dashes = 0;
                    }
                }
                continue;
            }

            for nc in chars.by_ref() {
                if nc == '>' {
                    break;
                }
            }
            continue;
        }

        result.push(c);
    }

    let decoded = result
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&ndash;", "–")
        .replace("&mdash;", "—");

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn get_feed_entries(app: &App, kind: DailyFeedKind) -> Vec<FeedEntry> {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return Vec::new(),
    };

    match kind {
        DailyFeedKind::News => {
            let mut entries = Vec::new();
            for item in &feed.news {
                if let Some(first_link) = item.links.first() {
                    let summary = item.story.as_deref().unwrap_or("");
                    let clean_story = strip_html_tags(summary);
                    let target = first_link.title.clone();
                    entries.push(FeedEntry {
                        title: if !clean_story.is_empty() {
                            clean_story
                        } else {
                            first_link.display_title().to_string()
                        },
                        target_article: target,
                        suffix: None,
                    });
                }
            }
            entries
        }
        DailyFeedKind::OnThisDay => {
            let mut entries = Vec::new();
            for event in &feed.onthisday {
                let target = event
                    .pages
                    .first()
                    .map(|p| p.title.clone())
                    .unwrap_or_default();
                if target.is_empty() {
                    continue;
                }
                let year_str = match event.year {
                    Some(y) if y < 0 => format!("{} BC", y.abs()),
                    Some(y) => format!("{}", y),
                    None => "—".to_string(),
                };
                let clean_text = strip_html_tags(&event.text);
                let display = format!("{}: {}", year_str, clean_text);
                entries.push(FeedEntry {
                    title: display,
                    target_article: target,
                    suffix: None,
                });
            }
            entries
        }
        DailyFeedKind::MostRead => {
            let mut entries = Vec::new();
            if let Some(payload) = &feed.mostread {
                for (idx, item) in payload.articles.iter().take(25).enumerate() {
                    let rank_str = format!("{}. ", idx + 1);
                    let views_str = item.views.map(crate::api::stats::format_metric);
                    let display = format!("{}{}", rank_str, item.display_title());
                    entries.push(FeedEntry {
                        title: display,
                        target_article: item.title.clone(),
                        suffix: views_str.map(|v| format!(" {} views", v)),
                    });
                }
            }
            entries
        }
    }
}

pub fn compute_daily_feed_modal_area(container_rect: Rect) -> Rect {
    centered_rect(75, 65, container_rect)
}

const MONTH_NAMES: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september",
    "october", "november", "december",
];

fn today_date_str() -> String {
    let (_y, m, d) = crate::api::daily_feed::utc_today();
    let month_str = MONTH_NAMES
        .get(m.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    format!("{} {}", month_str, d)
}

pub fn render_daily_feed_modal(f: &mut Frame, app: &App, size: Rect) {
    let state = match &app.daily_feed_modal {
        Some(s) => s,
        None => return,
    };

    let (icon, title_text, accent_color) = match state.kind {
        DailyFeedKind::News => (
            if app.config.ui.icons { "󰋫" } else { "" },
            "in the news".to_string(),
            theme::TEAL,
        ),
        DailyFeedKind::OnThisDay => (
            if app.config.ui.icons { "󰃭" } else { "" },
            format!("on this day · {}", today_date_str()),
            theme::YELLOW,
        ),
        DailyFeedKind::MostRead => (
            if app.config.ui.icons { "󰄬" } else { "" },
            "most read".to_string(),
            theme::VIOLET,
        ),
    };

    let modal_area = compute_daily_feed_modal_area(size);
    let modal_block = render_modal_frame_at(
        f,
        modal_area,
        icon,
        &title_text,
        accent_color,
        app.config.ui.rounded_borders,
    );

    let entries = get_feed_entries(app, state.kind);
    let total = entries.len();
    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let selected_idx = state.cursor_idx.min(total.saturating_sub(1));

    let scroll = if total <= inner_height || inner_height == 0 {
        0
    } else {
        selected_idx
            .saturating_sub(inner_height / 2)
            .min(total.saturating_sub(inner_height))
    };

    let mut lines = Vec::new();
    if entries.is_empty() {
        let empty_msg = if app.daily_feed.is_none() {
            "  loading daily feed from Wikipedia..."
        } else {
            "  no entries found."
        };
        lines.push(Line::from(vec![Span::styled(
            empty_msg,
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        let avail_w = (modal_area.width as usize).saturating_sub(8);
        for (idx, entry) in entries.iter().enumerate() {
            let is_selected = idx == selected_idx;
            let title = if entry.title.chars().count() > avail_w && avail_w > 3 {
                let byte_end = entry
                    .title
                    .char_indices()
                    .nth(avail_w.saturating_sub(3))
                    .map(|(i, _)| i)
                    .unwrap_or(entry.title.len());
                format!("{}...", &entry.title[..byte_end])
            } else {
                entry.title.clone()
            };

            lines.push(create_selectable_line(
                &title,
                is_selected,
                true,
                accent_color,
                entry.suffix.as_deref(),
            ));
        }
    }

    let p = Paragraph::new(lines)
        .block(modal_block)
        .scroll((scroll as u16, 0));

    f.render_widget(p, modal_area);
}
