use super::utils::{centered_rect, create_selectable_line, render_modal_frame_at};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
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
    pub link_idx: usize,
}

impl Default for DailyFeedModalState {
    fn default() -> Self {
        Self {
            kind: DailyFeedKind::News,
            cursor_idx: 0,
            link_idx: 0,
        }
    }
}

pub struct FeedEntry {
    pub title: String,
    pub target_article: String,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStyle {
    Normal,
    Bold,
    Italic,
    Link { link_idx: usize, title: String },
    BoldLink { link_idx: usize, title: String },
}

#[derive(Debug, Clone)]
pub struct StyledChunk {
    pub text: String,
    pub style: SpanStyle,
}

pub fn parse_story_html(input: &str) -> (Vec<StyledChunk>, Vec<String>) {
    let mut chunks = Vec::new();
    let mut links = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current_text = String::new();
    let mut in_bold = false;
    let mut in_italic = false;
    let mut current_link: Option<(usize, String)> = None;

    let flush = |chunks: &mut Vec<StyledChunk>,
                 current_text: &mut String,
                 in_bold: bool,
                 in_italic: bool,
                 current_link: &Option<(usize, String)>| {
        if current_text.is_empty() {
            return;
        }
        let style = match (current_link, in_bold, in_italic) {
            (Some((l_idx, target)), true, _) => SpanStyle::BoldLink {
                link_idx: *l_idx,
                title: target.clone(),
            },
            (Some((l_idx, target)), false, _) => SpanStyle::Link {
                link_idx: *l_idx,
                title: target.clone(),
            },
            (None, true, _) => SpanStyle::Bold,
            (None, false, true) => SpanStyle::Italic,
            (None, false, false) => SpanStyle::Normal,
        };
        let decoded = current_text
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#39;", "'")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&nbsp;", " ")
            .replace("&ndash;", "–")
            .replace("&mdash;", "—");
        chunks.push(StyledChunk {
            text: decoded,
            style,
        });
        current_text.clear();
    };

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

            let mut tag = String::new();
            for nc in chars.by_ref() {
                if nc == '>' {
                    break;
                }
                tag.push(nc);
            }

            let tag_lower = tag.to_lowercase();
            if tag_lower.starts_with("b") && !tag_lower.starts_with("br") {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                in_bold = true;
            } else if tag_lower == "/b" {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                in_bold = false;
            } else if tag_lower.starts_with("i") {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                in_italic = true;
            } else if tag_lower == "/i" {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                in_italic = false;
            } else if tag_lower.starts_with("a ") || tag_lower == "a" {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                let title = if let Some(pos) = tag.find("title=\"") {
                    let rest = &tag[pos + 7..];
                    rest.split('"').next().unwrap_or("").to_string()
                } else if let Some(pos) = tag.find("href=\"./") {
                    let rest = &tag[pos + 8..];
                    rest.split('"').next().unwrap_or("").replace('_', " ")
                } else {
                    String::new()
                };
                let l_idx = links.len();
                links.push(title.clone());
                current_link = Some((l_idx, title));
            } else if tag_lower == "/a" {
                flush(
                    &mut chunks,
                    &mut current_text,
                    in_bold,
                    in_italic,
                    &current_link,
                );
                current_link = None;
            }
            continue;
        }

        current_text.push(c);
    }

    flush(
        &mut chunks,
        &mut current_text,
        in_bold,
        in_italic,
        &current_link,
    );
    (chunks, links)
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

pub fn get_ongoing_links(ongoing: &[crate::api::daily_feed::OngoingItem]) -> Vec<(String, String)> {
    let mut links = Vec::new();
    for og in ongoing {
        links.push((og.target.clone(), og.display.clone()));
        for (sub_target, sub_display) in &og.sub_events {
            links.push((sub_target.clone(), sub_display.clone()));
        }
    }
    links
}

pub fn get_recent_deaths_links(
    deaths: &[crate::api::daily_feed::RecentDeathItem],
) -> Vec<(String, String)> {
    deaths
        .iter()
        .map(|d| (d.target.clone(), d.name.clone()))
        .collect()
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
            if !feed.ongoing.is_empty() {
                let first_target = feed
                    .ongoing
                    .first()
                    .map(|o| o.target.clone())
                    .unwrap_or_default();
                entries.push(FeedEntry {
                    title: "Ongoing".to_string(),
                    target_article: first_target,
                    suffix: None,
                });
            }
            if !feed.recent_deaths.is_empty() {
                let first_target = feed
                    .recent_deaths
                    .first()
                    .map(|d| d.target.clone())
                    .unwrap_or_default();
                entries.push(FeedEntry {
                    title: "Recent deaths".to_string(),
                    target_article: first_target,
                    suffix: None,
                });
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
                let display = format!("[ {} ]  {}", year_str, clean_text);
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
                        suffix: views_str.map(|v| format!("  {} views", v)),
                    });
                }
            }
            entries
        }
    }
}

pub fn compute_daily_feed_modal_area(container_rect: Rect, kind: DailyFeedKind) -> Rect {
    match kind {
        DailyFeedKind::MostRead => centered_rect(50, 57, container_rect),
        DailyFeedKind::News | DailyFeedKind::OnThisDay => centered_rect(75, 65, container_rect),
    }
}

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

fn today_date_str() -> String {
    let (_y, m, d) = crate::api::daily_feed::utc_today();
    let month_str = MONTH_NAMES
        .get(m.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    format!("{} {}", month_str, d)
}

pub fn wrap_story_spans(chunks: &[StyledChunk], max_width: usize) -> Vec<Vec<(String, SpanStyle)>> {
    let mut lines: Vec<Vec<(String, SpanStyle)>> = Vec::new();
    let mut current_line: Vec<(String, SpanStyle)> = Vec::new();
    let mut current_line_len = 0;
    let target_width = max_width.saturating_sub(4);

    for chunk in chunks {
        let mut word = String::new();
        for ch in chunk.text.chars() {
            if ch == ' ' {
                if !word.is_empty() {
                    let word_len = word.chars().count();
                    if current_line_len + word_len > target_width && current_line_len > 0 {
                        lines.push(current_line);
                        current_line = Vec::new();
                        current_line_len = 0;
                    }
                    current_line.push((word.clone(), chunk.style.clone()));
                    current_line_len += word_len;
                    word.clear();
                }
                if current_line_len > 0 && current_line_len < target_width {
                    current_line.push((" ".to_string(), chunk.style.clone()));
                    current_line_len += 1;
                }
            } else {
                word.push(ch);
            }
        }
        if !word.is_empty() {
            let word_len = word.chars().count();
            if current_line_len + word_len > target_width && current_line_len > 0 {
                lines.push(current_line);
                current_line = Vec::new();
                current_line_len = 0;
            }
            current_line.push((word, chunk.style.clone()));
            current_line_len += word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn normalize_search_str(s: &str) -> String {
    s.replace(['\u{a0}', '\u{202f}'], " ")
        .replace(['–', '—', '−'], "-")
}

pub fn parse_onthisday_event(
    text: &str,
    pages: &[crate::api::daily_feed::PageSummary],
) -> (Vec<StyledChunk>, Vec<String>) {
    let mut links = Vec::new();
    let mut match_targets: Vec<(String, String, usize)> = Vec::new();

    for page in pages {
        let canonical = &page.title;
        let display = page.display_title();
        let link_idx = links.len();
        links.push(canonical.clone());

        let norm_display = normalize_search_str(&display);
        let base_title = norm_display.split('(').next().unwrap_or(&norm_display).trim();
        if !base_title.is_empty() {
            match_targets.push((base_title.to_string(), canonical.clone(), link_idx));
            for suffix in [" line", " battle", " war", " siege", " treaty", " expedition"] {
                if let Some(stripped) = base_title.strip_suffix(suffix) {
                    if stripped.len() >= 3 {
                        match_targets.push((stripped.to_string(), canonical.clone(), link_idx));
                    }
                }
            }
        }
        if base_title != norm_display && !norm_display.is_empty() {
            match_targets.push((norm_display.to_string(), canonical.clone(), link_idx));
        }
    }

    match_targets.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
    let clean_text = normalize_search_str(text);

    let mut ranges: Vec<(usize, usize, usize, String)> = Vec::new();
    for (term, canonical, l_idx) in &match_targets {
        let term_lower = term.to_lowercase();
        let text_lower = clean_text.to_lowercase();

        let mut start = 0;
        while let Some(found_idx) = text_lower[start..].find(&term_lower) {
            let actual_start = start + found_idx;
            let actual_end = actual_start + term.len();

            let overlaps = ranges.iter().any(|(s, e, _, _)| {
                (actual_start >= *s && actual_start < *e)
                    || (actual_end > *s && actual_end <= *e)
                    || (actual_start <= *s && actual_end >= *e)
            });

            if !overlaps {
                ranges.push((actual_start, actual_end, *l_idx, canonical.clone()));
            }
            start = actual_start + 1;
            if start >= clean_text.len() {
                break;
            }
        }
    }

    ranges.sort_by_key(|r| r.0);

    let mut chunks = Vec::new();
    let mut last_idx = 0;
    for (start, end, l_idx, target) in ranges {
        if start > last_idx {
            chunks.push(StyledChunk {
                text: clean_text[last_idx..start].to_string(),
                style: SpanStyle::Normal,
            });
        }
        chunks.push(StyledChunk {
            text: clean_text[start..end].to_string(),
            style: SpanStyle::Link {
                link_idx: l_idx,
                title: target,
            },
        });
        last_idx = end;
    }
    if last_idx < clean_text.len() {
        chunks.push(StyledChunk {
            text: clean_text[last_idx..].to_string(),
            style: SpanStyle::Normal,
        });
    }

    if chunks.iter().all(|c| matches!(c.style, SpanStyle::Normal)) {
        if let Some(first_page) = pages.first() {
            chunks = vec![StyledChunk {
                text: clean_text,
                style: SpanStyle::Link {
                    link_idx: 0,
                    title: first_page.title.clone(),
                },
            }];
        }
    }

    (chunks, links)
}

pub fn render_daily_feed_modal(f: &mut Frame, app: &App, size: Rect) {
    let state = match &app.daily_feed_modal {
        Some(s) => s,
        None => return,
    };

    let (icon, title_text) = match state.kind {
        DailyFeedKind::News => (
            if app.config.ui.icons { "󰋫" } else { "" },
            "in the news".to_string(),
        ),
        DailyFeedKind::OnThisDay => (
            if app.config.ui.icons { "󰃭" } else { "" },
            format!("on this day · {}", today_date_str()),
        ),
        DailyFeedKind::MostRead => (
            if app.config.ui.icons { "󰄬" } else { "" },
            "most read".to_string(),
        ),
    };
    let accent_color = theme::BLUE;

    let modal_area = compute_daily_feed_modal_area(size, state.kind);
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

    if state.kind == DailyFeedKind::News {
        let feed = match &app.daily_feed {
            Some(f) => f,
            None => return,
        };

        let mut lines = Vec::new();
        if feed.news.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  no news stories available.",
                Style::default().fg(theme::GREY).italic(),
            )]));
        } else {
            let avail_w = (modal_area.width as usize).saturating_sub(4);
            let mut row_counter = 0;
            for item in &feed.news {
                let is_selected = row_counter == selected_idx;
                let raw_story = item.story.as_deref().unwrap_or("");
                let (chunks, story_links) = parse_story_html(raw_story);
                let active_link_idx = if is_selected {
                    state.link_idx.min(story_links.len().saturating_sub(1))
                } else {
                    0
                };

                let wrapped_lines = wrap_story_spans(&chunks, avail_w);
                for (line_idx, line_words) in wrapped_lines.into_iter().enumerate() {
                    let (prefix, prefix_style) = if line_idx == 0 {
                        if is_selected {
                            (" ▶ ", Style::default().fg(theme::BLUE).bold())
                        } else {
                            ("   ", Style::default().fg(theme::GREY))
                        }
                    } else {
                        ("   ", Style::default().fg(theme::GREY))
                    };

                    let mut spans = vec![Span::styled(prefix, prefix_style)];
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
                            SpanStyle::Link { link_idx, .. } => {
                                if is_selected && link_idx == active_link_idx {
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
                            SpanStyle::BoldLink { link_idx, .. } => {
                                if is_selected && link_idx == active_link_idx {
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
                row_counter += 1;
            }

            if !feed.ongoing.is_empty() || !feed.recent_deaths.is_empty() {
                let div_w = avail_w.saturating_sub(2);
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled("─".repeat(div_w), Style::default().fg(theme::DARK_GREY)),
                ]));
                lines.push(Line::from(""));
            }

            if !feed.ongoing.is_empty() {
                let is_selected = row_counter == selected_idx;
                let (prefix, prefix_style) = if is_selected {
                    (" ▶ ", Style::default().fg(theme::BLUE).bold())
                } else {
                    ("   ", Style::default().fg(theme::GREY))
                };
                let mut spans = vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled("Ongoing: ", Style::default().fg(theme::FG).bold()),
                ];

                let ongoing_links = get_ongoing_links(&feed.ongoing);
                let active_link_idx = if is_selected {
                    state.link_idx.min(ongoing_links.len().saturating_sub(1))
                } else {
                    0
                };

                let mut link_counter = 0;
                for (og_idx, og) in feed.ongoing.iter().enumerate() {
                    if og_idx > 0 {
                        spans.push(Span::styled(" · ", Style::default().fg(theme::GREY)));
                    }

                    let cur_l_idx = link_counter;
                    link_counter += 1;
                    let main_style = if is_selected && cur_l_idx == active_link_idx {
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
                    };
                    spans.push(Span::styled(og.display.clone(), main_style));

                    for (_sub_target, sub_display) in &og.sub_events {
                        let sub_l_idx = link_counter;
                        link_counter += 1;
                        spans.push(Span::styled(" (", Style::default().fg(theme::GREY)));
                        let sub_style = if is_selected && sub_l_idx == active_link_idx {
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
                        };
                        spans.push(Span::styled(sub_display.clone(), sub_style));
                        spans.push(Span::styled(")", Style::default().fg(theme::GREY)));
                    }
                }
                lines.push(Line::from(spans));
                lines.push(Line::from(""));
                row_counter += 1;
            }

            if !feed.recent_deaths.is_empty() {
                let is_selected = row_counter == selected_idx;
                let (prefix, prefix_style) = if is_selected {
                    (" ▶ ", Style::default().fg(theme::BLUE).bold())
                } else {
                    ("   ", Style::default().fg(theme::GREY))
                };
                let mut spans = vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled("Recent deaths: ", Style::default().fg(theme::FG).bold()),
                ];

                let active_link_idx = if is_selected {
                    state
                        .link_idx
                        .min(feed.recent_deaths.len().saturating_sub(1))
                } else {
                    0
                };

                for (d_idx, death) in feed.recent_deaths.iter().enumerate() {
                    if d_idx > 0 {
                        spans.push(Span::styled(" · ", Style::default().fg(theme::GREY)));
                    }
                    let death_style = if is_selected && d_idx == active_link_idx {
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
                    };
                    spans.push(Span::styled(death.name.clone(), death_style));
                }
                lines.push(Line::from(spans));
            }
        }

        let p = Paragraph::new(lines).block(modal_block);
        f.render_widget(p, modal_area);
        return;
    }

    if state.kind == DailyFeedKind::OnThisDay {
        let feed = match &app.daily_feed {
            Some(f) => f,
            None => return,
        };

        let mut lines = Vec::new();
        let mut line_offsets = Vec::new();
        if feed.onthisday.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  no historical milestones available.",
                Style::default().fg(theme::GREY).italic(),
            )]));
        } else {
            let avail_w = (modal_area.width as usize).saturating_sub(4);
            for (idx, event) in feed.onthisday.iter().enumerate() {
                line_offsets.push(lines.len());
                let is_selected = idx == selected_idx;
                let year_str = match event.year {
                    Some(y) if y < 0 => format!("{} BC", y.abs()),
                    Some(y) => format!("{}", y),
                    None => "—".to_string(),
                };
                let (chunks, event_links) = parse_onthisday_event(&event.text, &event.pages);
                let active_link_idx = if is_selected {
                    state.link_idx.min(event_links.len().saturating_sub(1))
                } else {
                    0
                };
                let badge_prefix = format!("[ {} ] ", year_str);
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
                            SpanStyle::Link { link_idx, .. } => {
                                if is_selected && link_idx == active_link_idx {
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
                            SpanStyle::BoldLink { link_idx, .. } => {
                                if is_selected && link_idx == active_link_idx {
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
        return;
    }

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
