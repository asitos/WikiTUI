use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use scraper::{ElementRef, Html, Node};

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub title: String,
    pub text: String,
    pub line_idx: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    pub lines: Vec<Line<'static>>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone)]
struct StyledToken {
    text: String,
    style: Style,
    link_target: Option<String>,
}

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
) {
    let tag_name = element.value().name();

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
        "h1" | "h2" => {
            current_style = current_style.fg(theme::RED).add_modifier(Modifier::BOLD);
        }
        "h3" | "h4" => {
            current_style = current_style.fg(theme::ORANGE).add_modifier(Modifier::BOLD);
        }
        "h5" | "h6" => {
            current_style = current_style.fg(theme::YELLOW).add_modifier(Modifier::BOLD);
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
            current_style = current_style
                .fg(theme::BLUE)
                .add_modifier(Modifier::UNDERLINED);
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
            current_tokens.push(StyledToken {
                text: "•".to_string(),
                style: Style::default().fg(theme::GREY),
                link_target: None,
            });
        }
        _ => {}
    }

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let cleaned_text = text.to_string();
                if !cleaned_text.is_empty() {
                    current_tokens.push(StyledToken {
                        text: cleaned_text,
                        style: current_style,
                        link_target: current_link.clone(),
                    });
                }
            }
            Node::Element(_) => {
                if let Some(child_ref) = ElementRef::wrap(child) {
                    process_element(
                        child_ref,
                        current_style,
                        current_link.clone(),
                        current_tokens,
                        doc,
                        max_width,
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

// extract title from wiki links
fn extract_title_from_href(href: &str) -> Option<String> {
    if href.starts_with("/wiki/") && !href.contains(':') {
        let title_part = href.trim_start_matches("/wiki/");
        let decoded = title_part.replace('_', " ");
        Some(decoded)
    } else {
        None
    }
}

fn wrap_and_append_block(tokens: &[StyledToken], doc: &mut ParsedDocument, max_width: usize) {
    let mut current_line_spans: Vec<Span<'static>> = Vec::new();
    let mut current_line_len = 0;

    for token in tokens {
        let words = token.text.split_inclusive(|c: char| c.is_whitespace());

        for word in words {
            if word.is_empty() {
                continue;
            }

            let word_len = word.chars().count();

            if current_line_len + word_len > max_width && current_line_len > 0 {
                let line_idx = doc.lines.len();
                doc.lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
                current_line_len = 0;

                if let Some(target) = &token.link_target {
                    doc.links.push(Link {
                        title: target.clone(),
                        text: word.trim().to_string(),
                        line_idx,
                        start_col: 0,
                        end_col: word_len,
                    });
                }
            } else if let Some(target) = &token.link_target {
                let start_col = current_line_len;
                let end_col = current_line_len + word_len;
                doc.links.push(Link {
                    title: target.clone(),
                    text: word.trim().to_string(),
                    line_idx: doc.lines.len(),
                    start_col,
                    end_col,
                });
            }

            let trimmed_word = word.to_string();
            current_line_spans.push(Span::styled(trimmed_word, token.style));
            current_line_len += word_len;
        }
    }

    if !current_line_spans.is_empty() {
        doc.lines.push(Line::from(current_line_spans));
    }
}
