use crate::parser::types::{ImageBlock, ParsedDocument, ParserContext};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use tl::{HTMLTag, Parser};

pub(crate) fn render_image_node(
    tag: &HTMLTag,
    parser: &Parser,
    doc: &mut ParsedDocument,
    ctx: &ParserContext,
) {
    if !ctx.show_images {
        return;
    }

    let (src, alt, width_px, height_px) = extract_image_attributes(tag, parser);
    let Some(url) = normalize_image_url(&src) else {
        return;
    };

    let caption = extract_caption(tag, parser).or(alt.clone());

    let max_cols = ctx.max_width.saturating_sub(4).max(10);
    let max_rows = ctx.max_image_height.max(5);

    let (cols, rows) = calculate_terminal_dimensions(width_px, height_px, max_cols, max_rows);

    let line_idx = doc.lines.len();

    let image_block = ImageBlock {
        url,
        alt,
        caption: caption.clone(),
        line_idx,
        height_lines: rows,
        width_cols: cols,
    };

    doc.lines.push(Line::from(""));
    for _ in 0..rows {
        doc.lines.push(Line::from(vec![
            Span::styled(" ".repeat(cols), Style::default())
        ]));
    }

    if let Some(cap) = caption {
        if !cap.trim().is_empty() {
            let cap_line = format!("▲ {}", cap.trim());
            doc.lines.push(Line::from(vec![
                Span::styled(cap_line, Style::default().fg(crate::theme::GREY).add_modifier(Modifier::ITALIC))
            ]));
        }
    }
    doc.lines.push(Line::from(""));

    doc.images.push(image_block);
}

fn normalize_image_url(src: &str) -> Option<String> {
    let clean = src.trim();
    if clean.is_empty() {
        return None;
    }

    if clean.contains("/static/images/") || clean.contains("red_pencile.svg") || clean.ends_with(".svg") {
        return None;
    }

    if clean.starts_with("//") {
        Some(format!("https:{}", clean))
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        Some(clean.to_string())
    } else if clean.starts_with('/') {
        Some(format!("https://en.wikipedia.org{}", clean))
    } else {
        None
    }
}

fn extract_image_attributes(
    tag: &HTMLTag,
    parser: &Parser,
) -> (String, Option<String>, Option<usize>, Option<usize>) {
    if tag.name().as_utf8_str() == "img" {
        return parse_img_tag(tag);
    }

    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(img_tag)) = child_handle.get(parser) {
            if img_tag.name().as_utf8_str() == "img" {
                return parse_img_tag(img_tag);
            }
        }
    }

    (String::new(), None, None, None)
}

fn parse_img_tag(tag: &HTMLTag) -> (String, Option<String>, Option<usize>, Option<usize>) {
    let src = tag
        .attributes()
        .get("src")
        .flatten()
        .map(|b| b.as_utf8_str().to_string())
        .unwrap_or_default();

    let alt = tag
        .attributes()
        .get("alt")
        .flatten()
        .map(|b| b.as_utf8_str().to_string())
        .filter(|s| !s.trim().is_empty());

    let width = tag
        .attributes()
        .get("width")
        .flatten()
        .and_then(|b| b.as_utf8_str().parse::<usize>().ok());

    let height = tag
        .attributes()
        .get("height")
        .flatten()
        .and_then(|b| b.as_utf8_str().parse::<usize>().ok());

    (src, alt, width, height)
}

fn extract_caption(tag: &HTMLTag, parser: &Parser) -> Option<String> {
    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(cap_tag)) = child_handle.get(parser) {
            if cap_tag.name().as_utf8_str() == "figcaption"
                || cap_tag
                    .attributes()
                    .get("class")
                    .flatten()
                    .map(|b| b.as_utf8_str().contains("thumbcaption"))
                    .unwrap_or(false)
            {
                let text = cap_tag.inner_text(parser).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn calculate_terminal_dimensions(
    w_px: Option<usize>,
    h_px: Option<usize>,
    max_cols: usize,
    max_rows: usize,
) -> (usize, usize) {
    if let (Some(w), Some(h)) = (w_px, h_px) {
        if w > 0 && h > 0 {
            let term_aspect = (w as f64) / (h as f64) * 2.0;
            let mut cols = (max_rows as f64 * term_aspect).round() as usize;
            cols = cols.clamp(10, max_cols);
            let rows = ((cols as f64) / term_aspect).round() as usize;
            let rows = rows.clamp(3, max_rows);
            return (cols, rows);
        }
    }
    (max_cols.min(40), max_rows.min(15))
}
