pub mod banners;
pub mod blocks;
pub mod codeblocks;
pub mod spoken;
pub mod tables;
pub mod types;
pub mod utils;

pub use banners::{ArticleBanner, BannerType};
pub use types::{AudioTrack, Heading, Link, ParsedDocument, SpokenAudio};
pub use utils::url_decode;

use crate::theme;
use blocks::wrap_and_append_block;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use types::StyledToken;
use utils::{decode_html_entities, extract_title_from_href, to_subscript_str, to_superscript_str};

fn is_references_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    let trimmed = lower.trim();
    trimmed == "references"
        || trimmed == "notes"
        || trimmed == "notes and references"
        || trimmed == "footnotes"
        || trimmed == "citations"
        || trimmed == "reference"
        || trimmed == "footnote"
        || trimmed == "citation"
        || trimmed.starts_with("references ")
        || trimmed.starts_with("notes ")
        || trimmed.starts_with("footnotes ")
}

fn is_references_id(id_str: &str) -> bool {
    let lower = id_str.to_lowercase();
    lower == "references"
        || lower == "notes"
        || lower == "notes_and_references"
        || lower == "footnotes"
        || lower == "citations"
        || lower.starts_with("cite_note")
        || lower.starts_with("cite_ref")
        || lower == "mw-references-wrap"
}

fn heading_info<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
) -> Option<(u8, String, Option<String>)> {
    let tag_name = tag.name().as_utf8_str();
    let id_attr = tag
        .attributes()
        .get("id")
        .flatten()
        .map(|b| decode_html_entities(&b.as_utf8_str()));

    if matches!(tag_name.as_ref(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        let level = tag_name
            .chars()
            .nth(1)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(1) as u8;
        let title = decode_html_entities(&tag.inner_text(parser))
            .trim()
            .to_string();
        return Some((level, title, id_attr));
    }

    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
    {
        if cls.contains("mw-heading") {
            for child_handle in tag.children().top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let child_name = child_tag.name().as_utf8_str();
                    if matches!(child_name.as_ref(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                        let level = child_name
                            .chars()
                            .nth(1)
                            .and_then(|c| c.to_digit(10))
                            .unwrap_or(1) as u8;
                        let title = decode_html_entities(&child_tag.inner_text(parser))
                            .trim()
                            .to_string();
                        let child_id = child_tag
                            .attributes()
                            .get("id")
                            .flatten()
                            .map(|b| decode_html_entities(&b.as_utf8_str()))
                            .or(id_attr);
                        return Some((level, title, child_id));
                    }
                }
            }
        }
    }

    None
}

pub fn parse_wikipedia_html(
    html: &str,
    max_width: usize,
    show_footnotes: bool,
    show_external_links: bool,
    heading_marker: bool,
    code_line_numbers: bool,
) -> ParsedDocument {
    let mut doc = ParsedDocument::default();
    let effective_width = max_width.max(10);

    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return doc;
    };
    let parser = dom.parser();
    let mut current_block_tokens: Vec<StyledToken> = Vec::new();
    let mut skipping_external_section = false;
    let mut skipping_references_section = false;

    for handle in dom.children() {
        if let Some(node) = handle.get(parser) {
            process_node(
                node,
                parser,
                Style::default().fg(theme::FG),
                None,
                &mut current_block_tokens,
                &mut doc,
                effective_width,
                None,
                show_footnotes,
                show_external_links,
                &mut skipping_external_section,
                &mut skipping_references_section,
                false,
                false,
                heading_marker,
                code_line_numbers,
            );
        }
    }

    if !current_block_tokens.is_empty() {
        wrap_and_append_block(&current_block_tokens, &mut doc, effective_width);
    }

    doc.plain_text_lower = doc
        .lines
        .iter()
        .map(|line| {
            let full_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            full_text.to_lowercase()
        })
        .collect();

    doc
}

#[allow(clippy::too_many_arguments)]
fn process_node<'a>(
    node: &'a tl::Node<'a>,
    parser: &'a tl::Parser<'a>,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    max_width: usize,
    list_item_idx: Option<usize>,
    show_footnotes: bool,
    show_external_links: bool,
    skipping_external_section: &mut bool,
    skipping_references_section: &mut bool,
    is_sup: bool,
    is_sub: bool,
    heading_marker: bool,
    code_line_numbers: bool,
) {
    match node {
        tl::Node::Raw(bytes) => {
            if *skipping_external_section || *skipping_references_section {
                return;
            }
            let raw_text = bytes.as_utf8_str();
            let decoded_text = decode_html_entities(&raw_text);
            let transformed = if is_sup {
                to_superscript_str(&decoded_text)
            } else if is_sub {
                to_subscript_str(&decoded_text)
            } else {
                decoded_text
            };
            let cleaned_text = transformed.replace(['\n', '\r', '\t'], " ");
            if !cleaned_text.trim().is_empty() {
                current_tokens.push(StyledToken {
                    text: cleaned_text,
                    style: parent_style,
                    link_target: parent_link,
                });
            }
        }
        tl::Node::Tag(tag) => {
            let tag_name_cow = tag.name().as_utf8_str();
            let tag_name = tag_name_cow.as_ref();

            if matches!(
                tag_name,
                "style"
                    | "script"
                    | "noscript"
                    | "head"
                    | "template"
                    | "link"
                    | "meta"
                    | "annotation"
            ) {
                return;
            }

            let class_attr = tag
                .attributes()
                .get("class")
                .flatten()
                .map(|b| decode_html_entities(&b.as_utf8_str()));
            let id_attr = tag
                .attributes()
                .get("id")
                .flatten()
                .map(|b| decode_html_entities(&b.as_utf8_str()));

            if spoken::is_spoken_wikipedia_tag(tag, parser) {
                if let Some(spoken_audio) = spoken::extract_spoken_audio(tag, parser) {
                    if doc.spoken_audio.is_none() {
                        doc.spoken_audio = Some(spoken_audio);
                    }
                }
                return;
            }

            if let Some((level, title, id_opt)) = heading_info(tag, parser) {
                let lower_title = title.to_lowercase();
                let lower_id = id_opt.as_deref().unwrap_or("").to_lowercase();

                let is_ext = !show_external_links
                    && (lower_title.starts_with("external link")
                        || lower_title.starts_with("external_link")
                        || lower_id == "external_links"
                        || lower_id == "external-links"
                        || lower_id == "externallinks");

                let is_refs = !show_footnotes
                    && (is_references_heading(&title) || is_references_id(&lower_id));

                if is_ext {
                    *skipping_external_section = true;
                    *skipping_references_section = false;
                    return;
                }
                if is_refs {
                    *skipping_references_section = true;
                    *skipping_external_section = false;
                    return;
                }

                if level <= 2 {
                    *skipping_external_section = false;
                    *skipping_references_section = false;
                } else if *skipping_external_section || *skipping_references_section {
                    return;
                }
            } else if *skipping_external_section || *skipping_references_section {
                return;
            }

            if !show_external_links {
                if let Some(ref id_str) = id_attr {
                    let lower = id_str.to_lowercase();
                    if lower == "external_links"
                        || lower == "external-links"
                        || lower == "externallinks"
                    {
                        *skipping_external_section = true;
                        return;
                    }
                }
            }

            if !show_footnotes {
                if let Some(ref id_str) = id_attr {
                    if is_references_id(id_str) {
                        if !id_str.starts_with("cite_note") && !id_str.starts_with("cite_ref") {
                            *skipping_references_section = true;
                        }
                        return;
                    }
                }
            }

            if let Some(ref class_str) = class_attr {
                if let Some(banner_type) = banners::classify_ambox_class(class_str.as_ref()) {
                    let final_message = banners::clean_ambox_text(&tag.inner_text(parser));
                    if !final_message.is_empty() {
                        if !current_tokens.is_empty() {
                            wrap_and_append_block(current_tokens, doc, max_width);
                            current_tokens.clear();
                        }

                        let color = banner_type.color();
                        let label = banner_type.label();

                        let side_margin = if max_width > 60 {
                            (max_width * 10 / 100).clamp(4, 20)
                        } else {
                            2
                        };
                        let left_padding = " ".repeat(side_margin);

                        let box_width = max_width.saturating_sub(side_margin * 2).max(20);
                        let header_str = format!("─ ⚠️ {} ", label);
                        let header_chars = header_str.chars().count();
                        let fill_top = box_width.saturating_sub(2 + header_chars);

                        doc.lines.push(Line::from(vec![
                            Span::raw(left_padding.clone()),
                            Span::styled("┌", Style::default().fg(color)),
                            Span::styled(
                                header_str,
                                Style::default().fg(color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled("─".repeat(fill_top), Style::default().fg(color)),
                            Span::styled("┐", Style::default().fg(color)),
                        ]));

                        let inner_width = box_width.saturating_sub(4).max(10);
                        let mut current_line = String::new();
                        for word in final_message.split_whitespace() {
                            if current_line.is_empty() {
                                current_line.push_str(word);
                            } else if current_line.chars().count() + 1 + word.chars().count()
                                <= inner_width
                            {
                                current_line.push(' ');
                                current_line.push_str(word);
                            } else {
                                let msg_len = current_line.chars().count();
                                let padding = inner_width.saturating_sub(msg_len);
                                doc.lines.push(Line::from(vec![
                                    Span::raw(left_padding.clone()),
                                    Span::styled("│ ", Style::default().fg(color)),
                                    Span::styled(
                                        current_line,
                                        Style::default()
                                            .fg(theme::FG)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                    Span::styled(
                                        " ".repeat(padding),
                                        Style::default().fg(theme::FG),
                                    ),
                                    Span::styled(" │", Style::default().fg(color)),
                                ]));
                                current_line = word.to_string();
                            }
                        }
                        if !current_line.is_empty() {
                            let msg_len = current_line.chars().count();
                            let padding = inner_width.saturating_sub(msg_len);
                            doc.lines.push(Line::from(vec![
                                Span::raw(left_padding.clone()),
                                Span::styled("│ ", Style::default().fg(color)),
                                Span::styled(
                                    current_line,
                                    Style::default()
                                        .fg(theme::FG)
                                        .add_modifier(Modifier::ITALIC),
                                ),
                                Span::styled(" ".repeat(padding), Style::default().fg(theme::FG)),
                                Span::styled(" │", Style::default().fg(color)),
                            ]));
                        }

                        doc.lines.push(Line::from(vec![
                            Span::raw(left_padding),
                            Span::styled("└", Style::default().fg(color)),
                            Span::styled(
                                "─".repeat(box_width.saturating_sub(2)),
                                Style::default().fg(color),
                            ),
                            Span::styled("┘", Style::default().fg(color)),
                        ]));
                        doc.lines.push(Line::from(""));
                    }
                    return;
                }
            }

            if let Some(ref class_str) = class_attr {
                if class_str.split_whitespace().any(|cls| {
                    matches!(
                        cls,
                        "sidebar"
                            | "infobox"
                            | "navbox"
                            | "mw-empty-elt"
                            | "noprint"
                            | "hatnote"
                            | "mw-jump-link"
                            | "catlinks"
                            | "vector-menu"
                            | "asbox"
                            | "stub"
                            | "cite-accessibility-label"
                            | "visually-hidden"
                            | "sr-only"
                    ) || (!show_footnotes
                        && matches!(
                            cls,
                            "reference"
                                | "reflist"
                                | "references"
                                | "mw-references-wrap"
                                | "references-wrap"
                        ))
                }) {
                    return;
                }
            }

            if !show_footnotes {
                if let Some(ref id_str) = id_attr {
                    if id_str.starts_with("cite_note")
                        || id_str.starts_with("cite_ref")
                        || id_str == "mw-references-wrap"
                    {
                        return;
                    }
                }
            }

            if tag_name == "table" {
                if !current_tokens.is_empty() {
                    wrap_and_append_block(current_tokens, doc, max_width);
                    current_tokens.clear();
                }
                tables::render_table(tag, parser, doc, max_width, show_footnotes);
                return;
            }

            if tag_name == "pre" {
                if !current_tokens.is_empty() {
                    wrap_and_append_block(current_tokens, doc, max_width);
                    current_tokens.clear();
                }
                let lang = codeblocks::extract_language(tag);
                codeblocks::render_code_block(tag, parser, doc, max_width, lang, code_line_numbers);
                return;
            }

            if let Some(ref class_str) = class_attr {
                if class_str.contains("mw-highlight") {
                    if !current_tokens.is_empty() {
                        wrap_and_append_block(current_tokens, doc, max_width);
                        current_tokens.clear();
                    }
                    let lang = codeblocks::extract_language(tag);
                    for child_handle in tag.children().top().iter() {
                        if let Some(tl::Node::Tag(pre_tag)) = child_handle.get(parser) {
                            if pre_tag.name().as_utf8_str() == "pre" {
                                codeblocks::render_code_block(
                                    pre_tag,
                                    parser,
                                    doc,
                                    max_width,
                                    lang,
                                    code_line_numbers,
                                );
                                return;
                            }
                        }
                    }
                }
            }

            let is_block_element = matches!(
                tag_name,
                "p" | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "li"
                    | "ul"
                    | "ol"
                    | "div"
                    | "section"
                    | "blockquote"
            );

            if is_block_element && !current_tokens.is_empty() {
                wrap_and_append_block(current_tokens, doc, max_width);
                current_tokens.clear();
            }

            if let Some(ref id_str) = id_attr {
                if id_str.starts_with("cite_note") || id_str.starts_with("cite_ref") {
                    doc.reference_targets
                        .entry(id_str.to_string())
                        .or_insert_with(|| doc.lines.len());
                }
            }

            let mut current_style = parent_style;
            let mut current_link = parent_link.clone();

            match tag_name {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = tag_name
                        .chars()
                        .nth(1)
                        .and_then(|c| c.to_digit(10))
                        .unwrap_or(1) as u8;
                    let title = decode_html_entities(&tag.inner_text(parser))
                        .trim()
                        .to_string();
                    if !title.is_empty() {
                        doc.headings.push(Heading {
                            title,
                            level,
                            line_idx: doc.lines.len(),
                        });
                    }

                    current_style = match tag_name {
                        "h1" | "h2" => current_style.fg(theme::RED).add_modifier(Modifier::BOLD),
                        "h3" | "h4" => current_style.fg(theme::ORANGE).add_modifier(Modifier::BOLD),
                        _ => current_style.fg(theme::YELLOW).add_modifier(Modifier::BOLD),
                    };

                    if heading_marker {
                        current_tokens.push(StyledToken {
                            text: "▍".to_string(),
                            style: current_style,
                            link_target: None,
                        });
                    }
                }
                "b" | "strong" => {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                "i" | "em" => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                "code" | "kbd" | "samp" | "tt" => {
                    current_style = current_style.fg(theme::ORANGE);
                }
                "abbr" => {
                    current_style = current_style.add_modifier(Modifier::UNDERLINED);
                }
                "a" => {
                    current_style = current_style.fg(theme::BLUE);
                    if let Some(href) = tag
                        .attributes()
                        .get("href")
                        .flatten()
                        .map(|b| b.as_utf8_str())
                    {
                        if let Some(title) = extract_title_from_href(href.as_ref()) {
                            let is_external =
                                title.starts_with("http://") || title.starts_with("https://");
                            if is_external {
                                current_style = current_style.fg(theme::TEAL);
                            }
                            current_link = Some(title);
                        }
                    } else if let Some(title_attr) = tag
                        .attributes()
                        .get("title")
                        .flatten()
                        .map(|b| decode_html_entities(&b.as_utf8_str()))
                    {
                        current_link = Some(title_attr);
                    }
                }
                "li" => {
                    let prefix = if let Some(idx) = list_item_idx {
                        format!("{}. ", idx)
                    } else {
                        "• ".to_string()
                    };
                    current_tokens.push(StyledToken {
                        text: prefix,
                        style: Style::default().fg(theme::GREY),
                        link_target: None,
                    });
                }
                "img" => {
                    let is_math_fallback = class_attr
                        .as_deref()
                        .map(|c| c.contains("mwe-math"))
                        .unwrap_or(false);

                    let alt_text = tag
                        .attributes()
                        .get("alt")
                        .flatten()
                        .or_else(|| tag.attributes().get("title").flatten())
                        .map(|b| decode_html_entities(&b.as_utf8_str()))
                        .unwrap_or_else(|| "image".to_string());

                    let clean_alt = alt_text.trim();
                    let is_latex = clean_alt.contains("\\displaystyle")
                        || clean_alt.contains("\\textstyle")
                        || clean_alt.starts_with("{\\");

                    if is_math_fallback || is_latex {
                        return;
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
                                extract_title_from_href(src.as_ref())
                            }
                        })
                        .or_else(|| current_link.clone());

                    current_tokens.push(StyledToken {
                        text: format!(" {} ", label),
                        style: Style::default()
                            .fg(theme::BEIGE)
                            .add_modifier(Modifier::ITALIC),
                        link_target: img_target,
                    });
                }
                _ => {}
            }

            let is_ordered_list = tag_name == "ol";
            let mut item_counter = 1;
            let current_is_sup = is_sup || tag_name == "sup";
            let current_is_sub = is_sub || tag_name == "sub";

            for child_handle in tag.children().top().iter() {
                if let Some(child_node) = child_handle.get(parser) {
                    let child_list_idx = if is_ordered_list {
                        if let tl::Node::Tag(child_tag) = child_node {
                            if child_tag.name().as_utf8_str() == "li" {
                                let idx = item_counter;
                                item_counter += 1;
                                Some(idx)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    process_node(
                        child_node,
                        parser,
                        current_style,
                        current_link.clone(),
                        current_tokens,
                        doc,
                        max_width,
                        child_list_idx,
                        show_footnotes,
                        show_external_links,
                        skipping_external_section,
                        skipping_references_section,
                        current_is_sup,
                        current_is_sub,
                        heading_marker,
                        code_line_numbers,
                    );
                }
            }

            if tag_name == "a" {
                if let Some(ref target) = current_link {
                    if target.starts_with("http://")
                        || target.starts_with("https://")
                        || target.starts_with("//")
                    {
                        let pill_text = if let Some(domain) = utils::extract_domain(target) {
                            format!(" ↗ {} ", domain)
                        } else {
                            " ↗ ".to_string()
                        };
                        current_tokens.push(StyledToken {
                            text: " ".to_string(),
                            style: Style::default(),
                            link_target: None,
                        });
                        current_tokens.push(StyledToken {
                            text: pill_text,
                            style: Style::default().fg(theme::GREY).bg(theme::LIGHT_BG),
                            link_target: current_link.clone(),
                        });
                    }
                }
            }

            if is_block_element && !current_tokens.is_empty() {
                wrap_and_append_block(current_tokens, doc, max_width);
                current_tokens.clear();
            }
        }
        tl::Node::Comment(_) => {}
    }
}
