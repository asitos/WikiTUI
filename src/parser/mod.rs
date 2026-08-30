pub mod banners;
pub mod blocks;
pub mod categories;
pub mod codeblocks;
pub mod elements;
pub(crate) mod node;
pub(crate) mod sections;
pub mod spoken;
pub mod tables;
pub mod types;
pub mod utils;

pub use banners::{ArticleBanner, BannerType};
pub use types::{AudioTrack, Heading, Link, ParsedDocument, SpokenAudio};
pub use utils::url_decode;

use blocks::wrap_and_append_block;
use node::process_node;
use ratatui::style::Style;
use types::StyledToken;

pub fn parse_wikipedia_html(
    html: &str,
    max_width: usize,
    show_footnotes: bool,
    show_external_links: bool,
    heading_marker: bool,
    code_line_numbers: bool,
    show_icons: bool,
) -> ParsedDocument {
    let mut doc = ParsedDocument::default();
    let effective_width = max_width.max(10);

    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return doc;
    };
    let parser = dom.parser();
    let mut ctx = types::ParserContext {
        parser,
        max_width: effective_width,
        show_footnotes,
        show_external_links,
        heading_marker,
        code_line_numbers,
        show_icons,
        skipping_external_section: false,
        skipping_references_section: false,
    };
    let mut current_block_tokens: Vec<StyledToken> = Vec::new();

    for handle in dom.children() {
        if let Some(node) = handle.get(parser) {
            process_node(
                node,
                &mut ctx,
                Style::default().fg(crate::theme::FG),
                None,
                &mut current_block_tokens,
                &mut doc,
                None,
                false,
                false,
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

    doc.links
        .sort_by_key(|l| l.span_indices.first().map(|(line, _)| *line).unwrap_or(0));
    debug_assert!(
        doc.links.windows(2).all(|w| {
            let l1 = w[0].span_indices.first().map(|(l, _)| *l).unwrap_or(0);
            let l2 = w[1].span_indices.first().map(|(l, _)| *l).unwrap_or(0);
            l1 <= l2
        }),
        "Link span indices must be monotonically sorted"
    );

    doc
}
