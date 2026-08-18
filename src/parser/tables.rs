#![allow(clippy::needless_range_loop)]

use super::types::{Link, ParsedDocument, StyledToken};
use super::utils::{decode_html_entities, extract_title_from_href};
use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

type CellLinkInfo = (String, String, Vec<(usize, usize)>);

#[derive(Debug, Clone)]
enum CellEntry {
    Origin {
        tokens: Vec<StyledToken>,
        colspan: usize,
        rowspan: usize,
        is_header: bool,
    },
    Covered {
        origin_r: usize,
        origin_c: usize,
    },
}

struct TableGrid {
    num_rows: usize,
    num_cols: usize,
    cells: Vec<Vec<CellEntry>>,
    caption: Option<String>,
}

pub fn render_table<'a>(
    table_tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    doc: &mut ParsedDocument,
    max_width: usize,
    show_footnotes: bool,
) {
    let grid = match parse_table_into_grid(table_tag, parser, show_footnotes) {
        Some(g) if g.num_rows > 0 && g.num_cols > 0 => g,
        _ => return,
    };

    let (num_rows, num_cols) = (grid.num_rows, grid.num_cols);

    if let Some(cap) = &grid.caption {
        doc.lines.push(Line::from(vec![
            Span::styled(" 📋 ", Style::default().fg(theme::VIOLET)),
            Span::styled(
                cap.clone(),
                Style::default()
                    .fg(theme::BEIGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let mut min_widths = vec![3usize; num_cols];
    let mut max_widths = vec![3usize; num_cols];

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin {
                tokens, colspan, ..
            } = &grid.cells[r][c]
            {
                let full_text: String = tokens.iter().map(|t| t.text.as_str()).collect();
                let longest_word = full_text
                    .split_whitespace()
                    .map(UnicodeWidthStr::width)
                    .max()
                    .unwrap_or(0);
                let total_width = UnicodeWidthStr::width(full_text.trim());

                if *colspan == 1 {
                    min_widths[c] = min_widths[c].max(longest_word.max(3));
                    max_widths[c] = max_widths[c].max(total_width.max(3));
                } else {
                    let share_min = (longest_word / *colspan).max(3);
                    let share_max = (total_width / *colspan).max(3);
                    for dc in 0..*colspan {
                        if c + dc < num_cols {
                            min_widths[c + dc] = min_widths[c + dc].max(share_min);
                            max_widths[c + dc] = max_widths[c + dc].max(share_max);
                        }
                    }
                }
            }
        }
    }

    let overhead = 3 * num_cols + 1;
    let available_width = max_width.saturating_sub(overhead).max(num_cols * 3);
    let total_max: usize = max_widths.iter().sum();
    let mut col_widths = vec![3usize; num_cols];

    if total_max <= available_width {
        for (i, w) in max_widths.iter().enumerate() {
            col_widths[i] = (*w).max(3);
        }
    } else {
        for i in 0..num_cols {
            let prop =
                (max_widths[i] as f64 / total_max as f64 * available_width as f64).round() as usize;
            col_widths[i] = prop.max(min_widths[i].min(15)).max(3);
        }
        let total_alloc: usize = col_widths.iter().sum();
        if total_alloc > available_width {
            let mut diff = total_alloc - available_width;
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

    let mut origin_lines = vec![vec![Vec::new(); num_cols]; num_rows];
    let mut origin_links = vec![vec![Vec::new(); num_cols]; num_rows];

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin {
                tokens, colspan, ..
            } = &grid.cells[r][c]
            {
                let mut span_w = (0..*colspan)
                    .filter_map(|dc| col_widths.get(c + dc))
                    .sum::<usize>();
                span_w += (*colspan - 1) * 3;
                let (lines, links) = wrap_cell_tokens(tokens, span_w);
                origin_lines[r][c] = lines;
                origin_links[r][c] = links;
            }
        }
    }

    let mut row_heights = vec![1usize; num_rows];
    for r in 0..num_rows {
        let mut max_h = 1;
        for c in 0..num_cols {
            if let CellEntry::Origin { rowspan, .. } = &grid.cells[r][c] {
                if *rowspan == 1 {
                    max_h = max_h.max(origin_lines[r][c].len());
                }
            }
        }
        row_heights[r] = max_h;
    }

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin { rowspan, .. } = &grid.cells[r][c] {
                if *rowspan > 1 {
                    let needed = origin_lines[r][c].len();
                    let current_total: usize =
                        (0..*rowspan).filter_map(|dr| row_heights.get(r + dr)).sum();
                    if needed > current_total {
                        let end_row = (r + *rowspan - 1).min(num_rows - 1);
                        row_heights[end_row] += needed - current_total;
                    }
                }
            }
        }
    }

    let mut cell_rendered_lines = vec![vec![Vec::new(); num_cols]; num_rows];
    let mut cell_rendered_links = vec![vec![Vec::new(); num_cols]; num_rows];

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin { rowspan, .. } = &grid.cells[r][c] {
                let all_lines = &origin_lines[r][c];
                let all_links = &origin_links[r][c];

                if *rowspan == 1 {
                    cell_rendered_lines[r][c] = all_lines.clone();
                    cell_rendered_links[r][c] = all_links.clone();
                } else {
                    let total_slots: usize =
                        (0..*rowspan).filter_map(|dr| row_heights.get(r + dr)).sum();
                    let top_pad = total_slots.saturating_sub(all_lines.len()) / 2;
                    let mut cursor = 0;
                    let mut slot = 0;

                    for dr in 0..*rowspan {
                        let curr_r = r + dr;
                        if curr_r >= num_rows {
                            break;
                        }
                        let mut chunk = Vec::new();
                        let mut chunk_links = Vec::new();

                        for local_line in 0..row_heights[curr_r] {
                            if slot >= top_pad && cursor < all_lines.len() {
                                chunk.push(all_lines[cursor].clone());
                                for (target, text, coords) in all_links {
                                    let matched: Vec<_> = coords
                                        .iter()
                                        .filter(|(src_l, _)| *src_l == cursor)
                                        .map(|(_, src_s)| (local_line, *src_s))
                                        .collect();
                                    if !matched.is_empty() {
                                        chunk_links.push((target.clone(), text.clone(), matched));
                                    }
                                }
                                cursor += 1;
                            } else {
                                chunk.push(Vec::new());
                            }
                            slot += 1;
                        }
                        cell_rendered_lines[curr_r][c] = chunk;
                        cell_rendered_links[curr_r][c] = chunk_links;
                    }
                }
            }
        }
    }

    let border_style = Style::default().fg(theme::DARK_GREY);

    let mut top_spans = vec![Span::styled("┌", border_style)];
    let mut c = 0;
    while c < num_cols {
        let colspan = match &grid.cells[0][c] {
            CellEntry::Origin { colspan, .. } => *colspan,
            _ => 1,
        };
        let mut span_w = (0..colspan)
            .filter_map(|dc| col_widths.get(c + dc))
            .sum::<usize>();
        span_w += (colspan - 1) * 3;
        top_spans.push(Span::styled("─".repeat(span_w + 2), border_style));
        c += colspan;
        if c < num_cols {
            top_spans.push(Span::styled("┬", border_style));
        }
    }
    top_spans.push(Span::styled("┐", border_style));
    doc.lines.push(Line::from(top_spans));

    for r in 0..num_rows {
        let start_idx = doc.lines.len();

        for line_in_row in 0..row_heights[r] {
            let mut line_spans = vec![Span::styled("│ ", border_style)];
            let mut c = 0;
            while c < num_cols {
                let (orig_c, colspan) = match &grid.cells[r][c] {
                    CellEntry::Origin { colspan, .. } => (c, *colspan),
                    CellEntry::Covered { origin_c, .. } => {
                        let span = match &grid.cells[r][*origin_c] {
                            CellEntry::Origin { colspan, .. } => *colspan,
                            _ => 1,
                        };
                        (*origin_c, span)
                    }
                };

                let mut span_w = (0..colspan)
                    .filter_map(|dc| col_widths.get(c + dc))
                    .sum::<usize>();
                span_w += (colspan - 1) * 3;

                let empty = Vec::new();
                let cell_spans = cell_rendered_lines
                    .get(r)
                    .and_then(|row| row.get(orig_c))
                    .and_then(|lines| lines.get(line_in_row))
                    .unwrap_or(&empty);

                let content_len: usize = cell_spans
                    .iter()
                    .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                    .sum();
                let padding = span_w.saturating_sub(content_len);

                for span in cell_spans {
                    line_spans.push(span.clone());
                }
                if padding > 0 {
                    line_spans.push(Span::raw(" ".repeat(padding)));
                }

                c += colspan;
                if c < num_cols {
                    line_spans.push(Span::styled(" │ ", border_style));
                } else {
                    line_spans.push(Span::styled(" │", border_style));
                }
            }
            doc.lines.push(Line::from(line_spans));
        }

        for col_i in 0..num_cols {
            if let Some(links) = cell_rendered_links.get(r).and_then(|row| row.get(col_i)) {
                for (target, text, coords) in links {
                    let mut span_indices = Vec::new();
                    for (local_l, local_s) in coords {
                        let abs_l = start_idx + local_l;
                        let mut offset = 1;
                        let mut prev_c = 0;
                        while prev_c < col_i {
                            let (orig_c, prev_span) = match &grid.cells[r][prev_c] {
                                CellEntry::Origin { colspan, .. } => (prev_c, *colspan),
                                CellEntry::Covered { origin_c, .. } => (*origin_c, 1),
                            };
                            let spans_len = cell_rendered_lines
                                .get(r)
                                .and_then(|row| row.get(orig_c))
                                .and_then(|lines| lines.get(*local_l))
                                .map(|s| s.len())
                                .unwrap_or(0);
                            offset += spans_len + 2;
                            prev_c += prev_span;
                        }
                        span_indices.push((abs_l, offset + local_s));
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
        }

        if r + 1 < num_rows {
            let is_header_sep = grid.cells[r].iter().any(|cell| match cell {
                CellEntry::Origin { is_header, .. } => *is_header,
                _ => false,
            });
            let sep_char = if is_header_sep { "═" } else { "─" };
            let mut sep_spans = vec![Span::styled("├", border_style)];

            let mut col = 0;
            while col < num_cols {
                let is_vert_cont = match &grid.cells[r + 1][col] {
                    CellEntry::Covered { origin_r, .. } => *origin_r <= r,
                    _ => false,
                };
                let colspan = match &grid.cells[r][col] {
                    CellEntry::Origin { colspan, .. } => *colspan,
                    CellEntry::Covered { origin_c, .. } => match &grid.cells[r][*origin_c] {
                        CellEntry::Origin { colspan, .. } => *colspan,
                        _ => 1,
                    },
                };
                let mut span_w = (0..colspan)
                    .filter_map(|dc| col_widths.get(col + dc))
                    .sum::<usize>();
                span_w += (colspan - 1) * 3;

                if is_vert_cont {
                    sep_spans.push(Span::raw(" ".repeat(span_w + 2)));
                } else {
                    sep_spans.push(Span::styled(sep_char.repeat(span_w + 2), border_style));
                }

                col += colspan;
                if col < num_cols {
                    let next_is_vert = match &grid.cells[r + 1][col] {
                        CellEntry::Covered { origin_r, .. } => *origin_r <= r,
                        _ => false,
                    };
                    let sep_joint = match (is_vert_cont, next_is_vert) {
                        (true, true) => "│",
                        (true, false) => "┤",
                        (false, true) => "├",
                        (false, false) => "┼",
                    };
                    sep_spans.push(Span::styled(sep_joint, border_style));
                }
            }
            sep_spans.push(Span::styled("┤", border_style));
            doc.lines.push(Line::from(sep_spans));
        }
    }

    let mut bot_spans = vec![Span::styled("└", border_style)];
    let mut c = 0;
    while c < num_cols {
        let colspan = match &grid.cells[num_rows - 1][c] {
            CellEntry::Origin { colspan, .. } => *colspan,
            _ => 1,
        };
        let mut span_w = (0..colspan)
            .filter_map(|dc| col_widths.get(c + dc))
            .sum::<usize>();
        span_w += (colspan - 1) * 3;
        bot_spans.push(Span::styled("─".repeat(span_w + 2), border_style));
        c += colspan;
        if c < num_cols {
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
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_line_len = 0;
    let mut links = Vec::new();

    for token in tokens {
        if token.text == "\n" {
            lines.push(current_line);
            current_line = Vec::new();
            current_line_len = 0;
            continue;
        }

        let mut link_coords = Vec::new();
        for word in token.text.split_inclusive(|c: char| c.is_whitespace()) {
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
                link_coords.push((lines.len(), current_line.len()));
            }
            current_line.push(Span::styled(word.to_string(), token.style));
            current_line_len += word_len;
        }

        if let Some(target) = &token.link_target {
            if !link_coords.is_empty() {
                links.push((target.clone(), token.text.trim().to_string(), link_coords));
            }
        }
    }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    (lines, links)
}

fn parse_table_into_grid<'a>(
    table_tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    show_footnotes: bool,
) -> Option<TableGrid> {
    let mut caption = None;
    let mut raw_tr_elements = Vec::new();

    for handle in table_tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = handle.get(parser) {
            match child_tag.name().as_utf8_str().as_ref() {
                "caption" => {
                    let text = child_tag.inner_text(parser);
                    let clean = decode_html_entities(&text).trim().to_string();
                    if !clean.is_empty() {
                        caption = Some(clean);
                    }
                }
                "tr" => raw_tr_elements.push(child_tag),
                "thead" | "tbody" | "tfoot" => {
                    for sub_handle in child_tag.children().top().iter() {
                        if let Some(tl::Node::Tag(sub_tag)) = sub_handle.get(parser) {
                            if sub_tag.name().as_utf8_str() == "tr" {
                                raw_tr_elements.push(sub_tag);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if raw_tr_elements.is_empty() {
        return None;
    }

    let mut grid: Vec<Vec<Option<CellEntry>>> = Vec::new();

    for (r, tr_tag) in raw_tr_elements.iter().enumerate() {
        if grid.len() <= r {
            grid.resize(r + 1, Vec::new());
        }
        let mut c = 0;

        for cell_handle in tr_tag.children().top().iter() {
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let name = cell_tag.name().as_utf8_str();
                let name_str = name.as_ref();
                if name_str == "th" || name_str == "td" {
                    let is_header = name_str == "th";
                    let colspan = cell_tag
                        .attributes()
                        .get("colspan")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse().ok())
                        .unwrap_or(1)
                        .max(1);
                    let rowspan = cell_tag
                        .attributes()
                        .get("rowspan")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse().ok())
                        .unwrap_or(1)
                        .max(1);

                    let default_style = if is_header {
                        Style::default()
                            .fg(theme::BEIGE)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::FG)
                    };

                    let mut tokens = Vec::new();
                    collect_cell_tokens(
                        cell_tag,
                        parser,
                        default_style,
                        None,
                        &mut tokens,
                        show_footnotes,
                    );

                    while c < grid[r].len() && grid[r][c].is_some() {
                        c += 1;
                    }

                    if grid.len() < r + rowspan {
                        grid.resize(r + rowspan, Vec::new());
                    }
                    for row_idx in r..(r + rowspan) {
                        if grid[row_idx].len() < c + colspan {
                            grid[row_idx].resize(c + colspan, None);
                        }
                    }

                    grid[r][c] = Some(CellEntry::Origin {
                        tokens,
                        colspan,
                        rowspan,
                        is_header,
                    });

                    for row_offset in 0..rowspan {
                        for col_offset in 0..colspan {
                            if row_offset == 0 && col_offset == 0 {
                                continue;
                            }
                            grid[r + row_offset][c + col_offset] =
                                Some(CellEntry::Covered { origin_r: r, origin_c: c });
                        }
                    }

                    c += colspan;
                }
            }
        }
    }

    let num_rows = grid.len();
    let num_cols = grid.iter().map(|row| row.len()).max().unwrap_or(0);

    if num_rows == 0 || num_cols == 0 {
        return None;
    }

    let clean_grid: Vec<Vec<CellEntry>> = grid
        .into_iter()
        .map(|mut row| {
            row.resize(num_cols, None);
            row.into_iter()
                .map(|opt| {
                    opt.unwrap_or_else(|| CellEntry::Origin {
                        tokens: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        is_header: false,
                    })
                })
                .collect()
        })
        .collect();

    Some(TableGrid {
        num_rows,
        num_cols,
        cells: clean_grid,
        caption,
    })
}

fn collect_cell_tokens<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    style: Style,
    link: Option<String>,
    tokens: &mut Vec<StyledToken>,
    show_footnotes: bool,
) {
    let tag_name = tag.name().as_utf8_str();
    let tag_name_str = tag_name.as_ref();
    if matches!(tag_name_str, "style" | "script" | "noscript") {
        return;
    }
    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
    {
        if !show_footnotes && cls.contains("reference") {
            return;
        }
    }

    let mut current_style = style;
    let mut current_link = link;

    match tag_name_str {
        "b" | "strong" | "th" => current_style = current_style.add_modifier(Modifier::BOLD),
        "i" | "em" => current_style = current_style.add_modifier(Modifier::ITALIC),
        "a" => {
            current_style = current_style.fg(theme::BLUE);
            if let Some(href) = tag
                .attributes()
                .get("href")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                if let Some(title) = extract_title_from_href(href.as_ref()) {
                    current_link = Some(title);
                }
            } else if let Some(title_attr) = tag
                .attributes()
                .get("title")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                current_link = Some(title_attr.to_string());
            }
        }
        "code" | "kbd" | "samp" | "tt" => current_style = current_style.fg(theme::ORANGE),
        "br" => tokens.push(StyledToken {
            text: "\n".to_string(),
            style: current_style,
            link_target: None,
        }),
        _ => {}
    }

    for child_handle in tag.children().top().iter() {
        if let Some(child_node) = child_handle.get(parser) {
            match child_node {
                tl::Node::Raw(bytes) => {
                    let raw_text = bytes.as_utf8_str();
                    let decoded_text = decode_html_entities(&raw_text);
                    let cleaned = decoded_text.replace(['\r', '\t', '\n'], " ");
                    if !cleaned.trim().is_empty() || cleaned == " " {
                        tokens.push(StyledToken {
                            text: cleaned,
                            style: current_style,
                            link_target: current_link.clone(),
                        });
                    }
                }
                tl::Node::Tag(child_tag) => {
                    collect_cell_tokens(
                        child_tag,
                        parser,
                        current_style,
                        current_link.clone(),
                        tokens,
                        show_footnotes,
                    );
                }
                _ => {}
            }
        }
    }
}
