#![allow(clippy::needless_range_loop)]

pub(crate) mod layout;
pub(crate) mod parser;
pub(crate) mod render;
pub(crate) mod types;
pub(crate) mod wrapping;

use crate::parser::types::ParsedDocument;
use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use layout::{compute_column_widths, compute_row_heights};
use parser::parse_table_into_grid;
use render::render_grid;
use types::CellEntry;
use wrapping::wrap_cell_tokens;

pub fn render_table<'a>(
    table_tag: &'a tl::HTMLTag<'a>,
    tl_parser: &'a tl::Parser<'a>,
    doc: &mut ParsedDocument,
    max_width: usize,
    show_footnotes: bool,
) {
    let grid = match parse_table_into_grid(table_tag, tl_parser, show_footnotes) {
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

    let col_widths = compute_column_widths(&grid, max_width);

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

    let row_heights = compute_row_heights(&grid, &origin_lines);

    render_grid(
        &grid,
        &col_widths,
        &row_heights,
        &origin_lines,
        &origin_links,
        doc,
    );
}
