pub mod banners;
pub mod blocks;
pub mod tables;
pub mod types;
pub mod utils;

pub use banners::{ArticleBanner, BannerType};
pub use types::{Heading, Link, ParsedDocument};
pub use utils::url_decode;

use crate::theme;
use blocks::wrap_and_append_block;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use scraper::{ElementRef, Html, Node};
use types::StyledToken;
use utils::extract_title_from_href;

pub fn parse_wikipedia_html(html: &str, max_width: usize) -> ParsedDocument {
    let fragment = Html::parse_fragment(html);
    let mut doc = ParsedDocument::default();
    let effective_width = max_width.max(10);

    let root = fragment.root_element();
    let mut current_block_tokens: Vec<StyledToken> = Vec::new();

    process_element(
        root,
        Style::default().fg(theme::FG),
        None,
        &mut current_block_tokens,
        &mut doc,
        effective_width,
        None,
    );

    if !current_block_tokens.is_empty() {
        wrap_and_append_block(&current_block_tokens, &mut doc, effective_width);
    }

    doc
}

fn process_element(
    element: ElementRef,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    max_width: usize,
    list_item_idx: Option<usize>,
) {
    let tag_name = element.value().name();

    // skip metadata tags
    if matches!(
        tag_name,
        "style" | "script" | "noscript" | "head" | "template" | "link" | "meta"
    ) {
        return;
    }

    // check if element is a Wikipedia ambox maintenance banner
    if let Some(class_attr) = element.value().attr("class") {
        if let Some(banner_type) = banners::classify_ambox_class(class_attr) {
            let raw_text = element.text().collect::<Vec<_>>().join(" ");
            let mut clean_text = raw_text.split_whitespace().collect::<Vec<_>>().join(" ");

            clean_text = clean_text
                .replace(" .", ".")
                .replace(" ,", ",")
                .replace(" :", ":")
                .replace(" ;", ";")
                .replace(" !", "!")
                .replace(" ?", "?");

            if let Some(idx) = clean_text.find("( Learn how") {
                clean_text.truncate(idx);
            }
            if let Some(idx) = clean_text.find("(Learn how") {
                clean_text.truncate(idx);
            }
            if let Some(open_paren) = clean_text.rfind('(') {
                let trailing = &clean_text[open_paren..];
                if trailing.contains("202") || trailing.contains("201") || trailing.contains("200")
                {
                    clean_text.truncate(open_paren);
                }
            }

            let final_message = clean_text.trim().to_string();
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
                    } else if current_line.chars().count() + 1 + word.chars().count() <= inner_width
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
                            Span::styled(" ".repeat(padding), Style::default().fg(theme::FG)),
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

    // skip wikipedia hidden elements
    if element.value().attr("class").is_some_and(|class_attr| {
        class_attr.split_whitespace().any(|cls| {
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
                    | "mw-cite-backlink"
            )
        })
    }) {
        return;
    }

    // render tables
    if tag_name == "table" {
        if !current_tokens.is_empty() {
            wrap_and_append_block(current_tokens, doc, max_width);
            current_tokens.clear();
        }
        tables::render_table(element, doc, max_width);
        return;
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

    let mut current_style = parent_style;
    let mut current_link = parent_link.clone();

    // styling for each tag
    match tag_name {
        // headings
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag_name
                .chars()
                .nth(1)
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as u8;
            let title = element.text().collect::<String>().trim().to_string();
            // store location of each heading
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
        }
        // bold
        "b" | "strong" => {
            current_style = current_style.add_modifier(Modifier::BOLD);
        }
        // italic
        "i" | "em" => {
            current_style = current_style.add_modifier(Modifier::ITALIC);
        }
        // links
        "a" => {
            current_style = current_style.fg(theme::BLUE);
            if let Some(href) = element.value().attr("href") {
                if let Some(title) = extract_title_from_href(href) {
                    current_link = Some(title);
                }
            } else if let Some(title_attr) = element.value().attr("title") {
                current_link = Some(title_attr.to_string());
            }
        }
        // lists
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
        // images
        "img" => {
            let alt_text = element
                .value()
                .attr("alt")
                .or_else(|| element.value().attr("title"))
                .unwrap_or("image");

            let clean_alt = alt_text.trim();
            let label = if clean_alt.is_empty() {
                "[image]".to_string()
            } else {
                format!("[image: {}]", clean_alt)
            };

            let img_target = element
                .value()
                .attr("src")
                .and_then(|src| {
                    if src.starts_with("//") {
                        Some(format!("https:{}", src))
                    } else if src.starts_with("http") {
                        Some(src.to_string())
                    } else {
                        extract_title_from_href(src)
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

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let cleaned_text = text.replace(['\n', '\r', '\t'], " ");
                if !cleaned_text.trim().is_empty() {
                    current_tokens.push(StyledToken {
                        text: cleaned_text,
                        style: current_style,
                        link_target: current_link.clone(),
                    });
                }
            }
            Node::Element(_) => {
                if let Some(child_ref) = ElementRef::wrap(child) {
                    let child_tag = child_ref.value().name();
                    let child_list_idx = if is_ordered_list && child_tag == "li" {
                        let idx = item_counter;
                        item_counter += 1;
                        Some(idx)
                    } else {
                        None
                    };

                    process_element(
                        child_ref,
                        current_style,
                        current_link.clone(),
                        current_tokens,
                        doc,
                        max_width,
                        child_list_idx,
                    );
                }
            }
            _ => {}
        }
    }

    if is_block_element && !current_tokens.is_empty() {
        wrap_and_append_block(current_tokens, doc, max_width);
        current_tokens.clear();
    }
}
