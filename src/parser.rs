use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use scraper::{ElementRef, Html, Node};

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub title: String,
    pub level: u8,
    pub line_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub title: String,
    pub text: String,
    pub line_idx: usize,
    pub span_indices: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    pub lines: Vec<Line<'static>>,
    pub links: Vec<Link>,
    pub headings: Vec<Heading>,
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

    // skip metadata tags
    if matches!(
        tag_name,
        "style" | "script" | "noscript" | "head" | "template" | "link" | "meta"
    ) {
        return;
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
            current_style = current_style
                .fg(theme::BLUE);
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
                text: "• ".to_string(),
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
        let mut link_span_indices = Vec::new();
        let mut current_link_line_idx = doc.lines.len();

        for word in words {
            if word.is_empty() {
                continue;
            }

            let word_len = word.chars().count();

            if current_line_len + word_len > max_width && current_line_len > 0 {
                if let Some(target) = token
                    .link_target
                    .as_ref()
                    .filter(|_| !link_span_indices.is_empty())
                {
                    doc.links.push(Link {
                        title: target.clone(),
                        text: token.text.trim().to_string(),
                        line_idx: current_link_line_idx,
                        span_indices: link_span_indices.clone(),
                    });
                    link_span_indices.clear();
                }

                doc.lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
                current_line_len = 0;
                current_link_line_idx = doc.lines.len();
            }

            if token.link_target.is_some() {
                link_span_indices.push(current_line_spans.len());
            }

            let trimmed_word = word.to_string();
            current_line_spans.push(Span::styled(trimmed_word, token.style));
            current_line_len += word_len;
        }

        if let Some(target) = token
            .link_target
            .as_ref()
            .filter(|_| !link_span_indices.is_empty())
        {
            doc.links.push(Link {
                title: target.clone(),
                text: token.text.trim().to_string(),
                line_idx: current_link_line_idx,
                span_indices: link_span_indices,
            });
        }
    }

    if !current_line_spans.is_empty() {
        doc.lines.push(Line::from(current_line_spans));
        doc.lines.push(Line::from(""));
    }
}
