use super::types::ParsedDocument;
use super::utils::decode_html_entities;
use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

pub(crate) fn pygments_class_style(class_str: &str) -> Style {
    let style = Style::default().fg(theme::FG);
    for cls in class_str.split_whitespace() {
        match cls {
            // keywords
            "k" | "kd" | "kn" | "kr" | "kc" | "ow" => {
                return style.fg(theme::RED).add_modifier(Modifier::BOLD);
            }
            // strings
            "s" | "s1" | "s2" | "sb" | "sc" | "sd" | "se" | "sh" | "si" | "sx" | "sr" | "ss" => {
                return style.fg(theme::LIME);
            }
            // comments
            "c" | "c1" | "cm" | "cs" | "cp" | "cpf" | "ch" => {
                return style.fg(theme::GREY).add_modifier(Modifier::ITALIC);
            }
            // functions / names / classes
            "nf" | "fm" | "nc" | "nn" | "ne" | "na" => {
                return style.fg(theme::BLUE).add_modifier(Modifier::BOLD);
            }
            // builtins / variables
            "nb" | "no" | "nv" | "vc" | "vg" | "vi" | "vm" => {
                return style.fg(theme::TEAL);
            }
            // numbers / literals
            "m" | "mi" | "mf" | "mh" | "mo" | "il" | "mb" => {
                return style.fg(theme::YELLOW);
            }
            // operators / punctuation
            "o" | "p" => {
                return style.fg(theme::GREY);
            }
            _ => {}
        }
    }
    style
}

pub(crate) fn extract_language(tag: &tl::HTMLTag) -> Option<String> {
    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
    {
        for word in cls.split_whitespace() {
            if let Some(lang) = word.strip_prefix("mw-highlight-lang-") {
                return Some(lang.to_string());
            }
            if let Some(lang) = word.strip_prefix("lang-") {
                return Some(lang.to_string());
            }
            if let Some(lang) = word.strip_prefix("language-") {
                return Some(lang.to_string());
            }
        }
    }
    None
}

fn collect_code_lines<'a>(
    node: &'a tl::Node<'a>,
    parser: &'a tl::Parser<'a>,
    current_style: Style,
    lines: &mut Vec<Vec<Span<'static>>>,
) {
    match node {
        tl::Node::Raw(b) => {
            let raw_text = b.as_utf8_str();
            let decoded = decode_html_entities(&raw_text);
            let parts: Vec<&str> = decoded.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    lines.push(Vec::new());
                }
                if !part.is_empty() {
                    if lines.is_empty() {
                        lines.push(Vec::new());
                    }
                    let last_idx = lines.len() - 1;
                    lines[last_idx].push(Span::styled(part.to_string(), current_style));
                }
            }
        }
        tl::Node::Tag(tag) => {
            let mut style = current_style;
            if let Some(cls) = tag
                .attributes()
                .get("class")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                let pygments = pygments_class_style(&cls);
                if pygments != Style::default().fg(theme::FG) {
                    style = pygments;
                }
            }
            for child_handle in tag.children().top().iter() {
                if let Some(child_node) = child_handle.get(parser) {
                    collect_code_lines(child_node, parser, style, lines);
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn render_code_block<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    doc: &mut ParsedDocument,
    max_width: usize,
    lang_opt: Option<String>,
    code_line_numbers: bool,
) {
    let mut raw_lines: Vec<Vec<Span<'static>>> = Vec::new();
    collect_code_lines(
        &tl::Node::Tag(tag.clone()),
        parser,
        Style::default().fg(theme::BEIGE),
        &mut raw_lines,
    );

    while let Some(last) = raw_lines.last() {
        if last.is_empty() || last.iter().all(|s| s.content.trim().is_empty()) {
            raw_lines.pop();
        } else {
            break;
        }
    }

    if raw_lines.is_empty() {
        return;
    }

    let border_color = theme::DARK_GREY;
    let effective_width = max_width.max(20);
    let (prefix_width, gutter_digits) = if code_line_numbers {
        let digits = if raw_lines.len() < 100 {
            2
        } else {
            raw_lines.len().to_string().len()
        };
        (2 + digits + 1, Some(digits))
    } else {
        (2, None)
    };
    let code_width = effective_width.saturating_sub(prefix_width);

    if let Some(ref lang) = lang_opt {
        let tag_text = format!(" {} ", lang);
        let tag_len = tag_text.len();
        let bar_len = effective_width.saturating_sub(tag_len + 4).max(2);
        doc.lines.push(Line::from(vec![
            Span::styled("┌─", Style::default().fg(border_color)),
            Span::styled(
                tag_text,
                Style::default()
                    .fg(theme::ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("─".repeat(bar_len), Style::default().fg(border_color)),
        ]));
    } else {
        doc.lines.push(Line::from(vec![
            Span::styled("┌", Style::default().fg(border_color)),
            Span::styled(
                "─".repeat(effective_width.saturating_sub(2).max(2)),
                Style::default().fg(border_color),
            ),
        ]));
    }

    for (line_idx, spans) in raw_lines.into_iter().enumerate() {
        let mut prefix_spans = vec![Span::styled("│ ", Style::default().fg(border_color))];
        if let Some(digits) = gutter_digits {
            let line_num_str = format!("{:>width$} ", line_idx + 1, width = digits);
            prefix_spans.push(Span::styled(
                line_num_str,
                Style::default().fg(theme::DARK_GREY),
            ));
        }

        let line_len: usize = spans
            .iter()
            .map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        if line_len <= code_width {
            let mut line_spans = prefix_spans;
            line_spans.extend(spans);
            doc.lines.push(Line::from(line_spans));
        } else {
            let mut cur_spans = prefix_spans;
            let mut cur_len = 0;
            for span in spans {
                let s_content = span.content;
                let s_style = span.style;
                let s_len = unicode_width::UnicodeWidthStr::width(s_content.as_ref());
                if cur_len + s_len <= code_width {
                    cur_spans.push(Span::styled(s_content.to_string(), s_style));
                    cur_len += s_len;
                } else {
                    for ch in s_content.chars() {
                        let ch_len = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                        if cur_len + ch_len > code_width && cur_len > 0 {
                            doc.lines.push(Line::from(cur_spans));
                            let mut cont_spans =
                                vec![Span::styled("│ ", Style::default().fg(border_color))];
                            if let Some(digits) = gutter_digits {
                                let continuation_gutter =
                                    format!("{:width$}↪ ", "", width = digits.saturating_sub(1));
                                cont_spans.push(Span::styled(
                                    continuation_gutter,
                                    Style::default().fg(theme::DARK_GREY),
                                ));
                            }
                            cur_spans = cont_spans;
                            cur_len = 0;
                        }
                        cur_spans.push(Span::styled(ch.to_string(), s_style));
                        cur_len += ch_len;
                    }
                }
            }
            if !cur_spans.is_empty() {
                doc.lines.push(Line::from(cur_spans));
            }
        }
    }

    doc.lines.push(Line::from(vec![
        Span::styled("└", Style::default().fg(border_color)),
        Span::styled(
            "─".repeat(effective_width.saturating_sub(2).max(2)),
            Style::default().fg(border_color),
        ),
    ]));
    doc.lines.push(Line::from(""));
}
