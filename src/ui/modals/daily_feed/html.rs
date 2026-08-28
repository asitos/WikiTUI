use super::types::{SpanStyle, StyledChunk};
use crate::parser::utils::decode_html_entities;

pub fn parse_story_html(input: &str) -> (Vec<StyledChunk>, Vec<String>) {
    let mut chunks = Vec::new();
    let mut links = Vec::new();
    let mut chars = input.chars();
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
        let decoded = decode_html_entities(current_text);
        chunks.push(StyledChunk {
            text: decoded,
            style,
        });
        current_text.clear();
    };

    while let Some(c) = chars.next() {
        if c == '<' {
            if chars.as_str().starts_with("!--") {
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

    let decoded = decode_html_entities(&result);

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
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
        let base_title = norm_display
            .split('(')
            .next()
            .unwrap_or(&norm_display)
            .trim();
        if !base_title.is_empty() {
            match_targets.push((base_title.to_string(), canonical.clone(), link_idx));
            for suffix in [
                " line",
                " battle",
                " war",
                " siege",
                " treaty",
                " expedition",
            ] {
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
