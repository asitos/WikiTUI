use super::types::{Heading, ParsedDocument, StyledToken};
use super::utils;
use crate::theme;
use ratatui::style::{Modifier, Style};

pub(crate) fn handle_heading_tag(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    tag_name: &str,
    doc: &mut ParsedDocument,
    current_tokens: &mut Vec<StyledToken>,
    heading_marker: bool,
    current_style: Style,
) -> Style {
    let level = tag_name
        .chars()
        .nth(1)
        .and_then(|c| c.to_digit(10))
        .unwrap_or(1) as u8;
    let title = utils::decode_html_entities(&tag.inner_text(parser))
        .trim()
        .to_string();
    if !title.is_empty() {
        doc.headings.push(Heading {
            title,
            level,
            line_idx: doc.lines.len(),
        });
    }

    let styled = match tag_name {
        "h1" | "h2" => current_style.fg(theme::RED).add_modifier(Modifier::BOLD),
        "h3" | "h4" => current_style.fg(theme::ORANGE).add_modifier(Modifier::BOLD),
        _ => current_style.fg(theme::YELLOW).add_modifier(Modifier::BOLD),
    };

    if heading_marker {
        current_tokens.push(StyledToken {
            text: "▍".to_string(),
            style: styled,
            link_target: None,
        });
    }

    styled
}

pub(crate) fn handle_img_tag(
    tag: &tl::HTMLTag,
    class_attr: Option<&str>,
    current_link: &Option<String>,
) -> Option<StyledToken> {
    let is_math_fallback = class_attr
        .map(|c| c.contains("mwe-math"))
        .unwrap_or(false);

    let alt_text = tag
        .attributes()
        .get("alt")
        .flatten()
        .or_else(|| tag.attributes().get("title").flatten())
        .map(|b| utils::decode_html_entities(&b.as_utf8_str()))
        .unwrap_or_else(|| "image".to_string());

    let clean_alt = alt_text.trim();
    let is_latex = clean_alt.contains("\\displaystyle")
        || clean_alt.contains("\\textstyle")
        || clean_alt.starts_with("{\\");

    if is_math_fallback || is_latex {
        return None;
    }

    let label = if clean_alt.is_empty() {
        "[image]".to_string()
    } else {
        format!("[image: {}]", clean_alt)
    };

    let img_target = tag
        .attributes()
        .get("src")
        .flatten()
        .map(|b| b.as_utf8_str())
        .and_then(|src| {
            if src.starts_with("//") {
                Some(format!("https:{}", src))
            } else if src.starts_with("http") {
                Some(src.to_string())
            } else {
                utils::extract_title_from_href(src.as_ref())
            }
        })
        .or_else(|| current_link.clone());

    Some(StyledToken {
        text: format!(" {} ", label),
        style: Style::default()
            .fg(theme::BEIGE)
            .add_modifier(Modifier::ITALIC),
        link_target: img_target,
    })
}

pub(crate) fn handle_external_link_pill(target: &str, current_link: &Option<String>) -> Vec<StyledToken> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("//")
    {
        let pill_text = if let Some(domain) = utils::extract_domain(target) {
            format!(" ↗ {} ", domain)
        } else {
            " ↗ ".to_string()
        };
        vec![
            StyledToken {
                text: " ".to_string(),
                style: Style::default(),
                link_target: None,
            },
            StyledToken {
                text: pill_text,
                style: Style::default().fg(theme::GREY).bg(theme::LIGHT_BG),
                link_target: current_link.clone(),
            },
        ]
    } else {
        Vec::new()
    }
}
