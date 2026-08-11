use super::types::{Link, ParsedDocument, StyledToken};
use ratatui::text::{Line, Span};

pub(crate) fn wrap_and_append_block(
    tokens: &[StyledToken],
    doc: &mut ParsedDocument,
    max_width: usize,
) {
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

            let word_len = unicode_width::UnicodeWidthStr::width(word);

            if current_line_len + word_len > max_width && current_line_len > 0 {
                doc.lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
                current_line_len = 0;
                current_link_line_idx = doc.lines.len();
            }

            if token.link_target.is_some() {
                link_span_indices.push((current_link_line_idx, current_line_spans.len()));
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
                span_indices: link_span_indices,
            });
        }
    }

    if !current_line_spans.is_empty() {
        doc.lines.push(Line::from(current_line_spans));
        doc.lines.push(Line::from(""));
    }
}
