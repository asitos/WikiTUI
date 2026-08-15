use super::types::{Link, ParsedDocument, StyledToken};
use super::utils::extract_title_from_href;
use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use scraper::{ElementRef, Node};
use unicode_width::UnicodeWidthStr;

type CellLinkInfo = (String, String, Vec<(usize, usize)>);

#[derive(Debug, Clone)]
struct TableCell {
    tokens: Vec<StyledToken>,
    colspan: usize,
}

#[derive(Debug, Clone)]
struct TableRow {
    cells: Vec<TableCell>,
    is_header: bool,
}

pub fn render_table(table_el: ElementRef, doc: &mut ParsedDocument, max_width: usize) {
    let (caption, rows) = extract_table_rows(table_el);
    if rows.is_empty() {
        return;
    }

    let num_cols = rows
        .iter()
        .map(|r| r.cells.iter().map(|c| c.colspan).sum::<usize>())
        .max()
        .unwrap_or(0);

    if num_cols == 0 {
        return;
    }

    if let Some(cap) = caption {
        doc.lines.push(Line::from(vec![
            Span::styled(" 📋 ", Style::default().fg(theme::VIOLET)),
            Span::styled(
                cap,
                Style::default().fg(theme::BEIGE).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let mut min_widths = vec![3usize; num_cols];
    let mut max_widths = vec![3usize; num_cols];

    for row in &rows {
        let mut col_idx = 0;
        for cell in &row.cells {
            let full_text: String = cell.tokens.iter().map(|t| t.text.as_str()).collect();
            let mut longest_word = 0;
            for word in full_text.split_whitespace() {
                longest_word = longest_word.max(UnicodeWidthStr::width(word));
            }
            let total_width = UnicodeWidthStr::width(full_text.trim());

            if cell.colspan == 1 {
                if col_idx < num_cols {
                    min_widths[col_idx] = min_widths[col_idx].max(longest_word.max(3));
                    max_widths[col_idx] = max_widths[col_idx].max(total_width.max(3));
                }
            } else {
                let share_min = (longest_word / cell.colspan).max(3);
                let share_max = (total_width / cell.colspan).max(3);
                for c in 0..cell.colspan {
                    if col_idx + c < num_cols {
                        min_widths[col_idx + c] = min_widths[col_idx + c].max(share_min);
                        max_widths[col_idx + c] = max_widths[col_idx + c].max(share_max);
                    }
                }
            }
            col_idx += cell.colspan;
        }
    }

    let overhead = 3 * num_cols + 1;
    let available_content_width = max_width.saturating_sub(overhead).max(num_cols * 3);

    let total_max: usize = max_widths.iter().sum();
    let mut col_widths = vec![3usize; num_cols];

    if total_max <= available_content_width {
        for (i, w) in max_widths.iter().enumerate() {
            col_widths[i] = (*w).max(3);
        }
    } else {
        let mut remaining = available_content_width;
        for i in 0..num_cols {
            let prop = (max_widths[i] as f64 / total_max as f64 * available_content_width as f64)
                .round() as usize;
            let allocated = prop.max(min_widths[i].min(15)).max(3);
            col_widths[i] = allocated;
            remaining = remaining.saturating_sub(allocated);
        }

        let total_allocated: usize = col_widths.iter().sum();
        if total_allocated > available_content_width {
            let mut diff = total_allocated - available_content_width;
            for i in (0..num_cols).rev() {
                if col_widths[i] > 3 {
                    let shrink = (col_widths[i] - 3).min(diff);
                    col_widths[i] -= shrink;
                    diff -= shrink;
                    if diff == 0 {
                        break;
                    }
                }
            }
        }
    }

    let border_style = Style::default().fg(theme::DARK_GREY);

    let mut top_spans = Vec::new();
    top_spans.push(Span::styled("┌", border_style));
    for (i, w) in col_widths.iter().enumerate() {
        top_spans.push(Span::styled("─".repeat(*w + 2), border_style));
        if i + 1 < num_cols {
            top_spans.push(Span::styled("┬", border_style));
        }
    }
    top_spans.push(Span::styled("┐", border_style));
    doc.lines.push(Line::from(top_spans));

    let rows_len = rows.len();
    for (row_idx, row) in rows.iter().enumerate() {
        let mut wrapped_cells: Vec<Vec<Vec<Span<'static>>>> = Vec::new();
        let mut cell_links_info: Vec<Vec<CellLinkInfo>> = Vec::new();
        let mut col_idx = 0;

        for cell in &row.cells {
            let span_width = if cell.colspan == 1 {
                if col_idx < num_cols {
                    col_widths[col_idx]
                } else {
                    3
                }
            } else {
                let mut w = 0;
                for c in 0..cell.colspan {
                    if col_idx + c < num_cols {
                        w += col_widths[col_idx + c];
                    }
                }
                w + (cell.colspan - 1) * 3
            };

            let (cell_lines, links) = wrap_cell_tokens(&cell.tokens, span_width);
            wrapped_cells.push(cell_lines);
            cell_links_info.push(links);

            col_idx += cell.colspan;
        }

        let max_cell_lines = wrapped_cells.iter().map(|lines| lines.len()).max().unwrap_or(1);
        let start_line_idx = doc.lines.len();

        for line_in_row in 0..max_cell_lines {
            let mut line_spans = Vec::new();
            line_spans.push(Span::styled("│ ", border_style));

            let mut cur_col = 0;
            for (c_idx, cell) in row.cells.iter().enumerate() {
                let span_width = if cell.colspan == 1 {
                    if cur_col < num_cols {
                        col_widths[cur_col]
                    } else {
                        3
                    }
                } else {
                    let mut w = 0;
                    for c in 0..cell.colspan {
                        if cur_col + c < num_cols {
                            w += col_widths[cur_col + c];
                        }
                    }
                    w + (cell.colspan - 1) * 3
                };

                let empty_vec = Vec::new();
                let cell_spans = wrapped_cells
                    .get(c_idx)
                    .and_then(|lines| lines.get(line_in_row))
                    .unwrap_or(&empty_vec);

                let content_len: usize = cell_spans
                    .iter()
                    .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                    .sum();

                let padding = span_width.saturating_sub(content_len);

                for span in cell_spans {
                    line_spans.push(span.clone());
                }
                if padding > 0 {
                    line_spans.push(Span::raw(" ".repeat(padding)));
                }

                if c_idx + 1 < row.cells.len() {
                    line_spans.push(Span::styled(" │ ", border_style));
                } else {
                    line_spans.push(Span::styled(" │", border_style));
                }

                cur_col += cell.colspan;
            }

            doc.lines.push(Line::from(line_spans));
        }

        for (c_idx, links) in cell_links_info.iter().enumerate() {
            for (target, text, span_coords) in links {
                let mut span_indices = Vec::new();
                for (local_line_idx, local_span_idx) in span_coords {
                    let absolute_line_idx = start_line_idx + local_line_idx;
                    let mut offset = 1;
                    for prev_c in 0..c_idx {
                        let prev_spans = wrapped_cells
                            .get(prev_c)
                            .and_then(|lines| lines.get(*local_line_idx))
                            .map(|s| s.len())
                            .unwrap_or(0);
                        let has_padding = 1;
                        let border = 1;
                        offset += prev_spans + has_padding + border;
                    }
                    span_indices.push((absolute_line_idx, offset + local_span_idx));
                }

                if !span_indices.is_empty() {
                    doc.links.push(Link {
                        title: target.clone(),
                        text: text.clone(),
                        span_indices,
                    });
                }
            }
        }

        if row_idx + 1 < rows_len {
            let sep_char = if row.is_header { "═" } else { "─" };
            let mut sep_spans = Vec::new();
            sep_spans.push(Span::styled("├", border_style));
            for (i, w) in col_widths.iter().enumerate() {
                sep_spans.push(Span::styled(sep_char.repeat(*w + 2), border_style));
                if i + 1 < num_cols {
                    sep_spans.push(Span::styled("┼", border_style));
                }
            }
            sep_spans.push(Span::styled("┤", border_style));
            doc.lines.push(Line::from(sep_spans));
        }
    }

    let mut bot_spans = Vec::new();
    bot_spans.push(Span::styled("└", border_style));
    for (i, w) in col_widths.iter().enumerate() {
        bot_spans.push(Span::styled("─".repeat(*w + 2), border_style));
        if i + 1 < num_cols {
            bot_spans.push(Span::styled("┴", border_style));
        }
    }
    bot_spans.push(Span::styled("┘", border_style));
    doc.lines.push(Line::from(bot_spans));
    doc.lines.push(Line::from(""));
}

fn wrap_cell_tokens(
    tokens: &[StyledToken],
    max_width: usize,
) -> (Vec<Vec<Span<'static>>>, Vec<CellLinkInfo>) {
    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_line_len = 0;
    let mut links = Vec::new();

    for token in tokens {
        if token.text == "\n" {
            lines.push(current_line);
            current_line = Vec::new();
            current_line_len = 0;
            continue;
        }

        let words = token.text.split_inclusive(|c: char| c.is_whitespace());
        let mut link_span_coords = Vec::new();

        for word in words {
            if word.is_empty() {
                continue;
            }

            let word_len = UnicodeWidthStr::width(word);

            if current_line_len + word_len > max_width && current_line_len > 0 {
                lines.push(current_line);
                current_line = Vec::new();
                current_line_len = 0;
            }

            if token.link_target.is_some() {
                link_span_coords.push((lines.len(), current_line.len()));
            }

            current_line.push(Span::styled(word.to_string(), token.style));
            current_line_len += word_len;
        }

        if let Some(target) = &token.link_target {
            if !link_span_coords.is_empty() {
                links.push((target.clone(), token.text.trim().to_string(), link_span_coords));
            }
        }
    }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    (lines, links)
}

fn extract_table_rows(table_el: ElementRef) -> (Option<String>, Vec<TableRow>) {
    let mut caption = None;
    let mut rows = Vec::new();

    for child in table_el.children() {
        if let Some(child_ref) = ElementRef::wrap(child) {
            let name = child_ref.value().name();
            if name == "caption" {
                let text = child_ref.text().collect::<Vec<_>>().join(" ");
                let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !clean.is_empty() {
                    caption = Some(clean);
                }
            } else if name == "tr" {
                if let Some(row) = extract_row(child_ref) {
                    rows.push(row);
                }
            } else if matches!(name, "thead" | "tbody" | "tfoot") {
                for subchild in child_ref.children() {
                    if let Some(subchild_ref) = ElementRef::wrap(subchild) {
                        if subchild_ref.value().name() == "tr" {
                            if let Some(row) = extract_row(subchild_ref) {
                                rows.push(row);
                            }
                        }
                    }
                }
            }
        }
    }

    (caption, rows)
}

fn extract_row(tr_el: ElementRef) -> Option<TableRow> {
    let mut cells = Vec::new();
    let mut all_headers = true;

    for child in tr_el.children() {
        if let Some(child_ref) = ElementRef::wrap(child) {
            let name = child_ref.value().name();
            if name == "th" || name == "td" {
                let is_header = name == "th";
                if !is_header {
                    all_headers = false;
                }

                let colspan = child_ref
                    .value()
                    .attr("colspan")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1)
                    .max(1);

                let default_style = if is_header {
                    Style::default().fg(theme::BEIGE).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::FG)
                };

                let mut tokens = Vec::new();
                collect_cell_tokens(child_ref, default_style, None, &mut tokens);

                cells.push(TableCell {
                    tokens,
                    colspan,
                });
            }
        }
    }

    if cells.is_empty() {
        None
    } else {
        Some(TableRow {
            is_header: all_headers,
            cells,
        })
    }
}

fn collect_cell_tokens(
    element: ElementRef,
    style: Style,
    link: Option<String>,
    tokens: &mut Vec<StyledToken>,
) {
    let tag = element.value().name();
    if matches!(tag, "style" | "script" | "noscript") {
        return;
    }

    if let Some(cls) = element.value().attr("class") {
        if cls.contains("reference") {
            return;
        }
    }

    let mut current_style = style;
    let mut current_link = link;

    match tag {
        "b" | "strong" | "th" => {
            current_style = current_style.add_modifier(Modifier::BOLD);
        }
        "i" | "em" => {
            current_style = current_style.add_modifier(Modifier::ITALIC);
        }
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
        "code" => {
            current_style = current_style.fg(theme::TEAL);
        }
        "br" => {
            tokens.push(StyledToken {
                text: "\n".to_string(),
                style: current_style,
                link_target: None,
            });
        }
        _ => {}
    }

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let cleaned = text.replace(['\r', '\t', '\n'], " ");
                if !cleaned.trim().is_empty() || cleaned == " " {
                    tokens.push(StyledToken {
                        text: cleaned,
                        style: current_style,
                        link_target: current_link.clone(),
                    });
                }
            }
            Node::Element(_) => {
                if let Some(child_ref) = ElementRef::wrap(child) {
                    collect_cell_tokens(child_ref, current_style, current_link.clone(), tokens);
                }
            }
            _ => {}
        }
    }
}
