use crate::app::{App, InputMode, PaneContent, TextSelection};
use ratatui::layout::Rect;

pub fn get_char_coord_in_article_pane(
    pane: &crate::app::Pane,
    rect: Rect,
    col: u16,
    row: u16,
) -> Option<(usize, usize)> {
    let PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
        return None;
    };

    if col < rect.x || col >= rect.x + rect.width || row < rect.y || row >= rect.y + rect.height {
        return None;
    }

    let inner_y = rect.y + 1;
    let inner_x = rect.x + 2;

    let line_offset = if row < inner_y {
        0
    } else {
        (row - inner_y) as usize
    };

    let line_idx = pane.scroll_offset + line_offset;
    let line_idx = line_idx.min(parsed_doc.lines.len().saturating_sub(1));

    let char_col = if col < inner_x {
        0
    } else {
        (col - inner_x) as usize
    };

    Some((line_idx, char_col))
}

pub fn handle_selection_down(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) -> bool {
    if app.input_mode != InputMode::Normal {
        return false;
    }

    if row == 0 || row >= term_height.saturating_sub(1) {
        return false;
    }

    let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
    let tab = app.active_tab_mut();
    let rects = tab.layout_root.compute_rects(main_rect);

    for (pane_idx, rect) in rects {
        if col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
        {
            tab.active_pane_idx = pane_idx;
            let pane = &mut tab.panes[pane_idx];
            if matches!(pane.content, PaneContent::ArticleText { .. }) {
                if let Some(coord) = get_char_coord_in_article_pane(pane, rect, col, row) {
                    pane.text_selection = None;
                    pane.selection_anchor = Some(coord);
                    pane.is_mouse_selecting = true;
                    return true;
                }
            }
        }
    }
    false
}

pub fn handle_selection_drag(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) -> bool {
    if app.input_mode != InputMode::Normal {
        return false;
    }

    let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
    let tab = app.active_tab_mut();
    let rects = tab.layout_root.compute_rects(main_rect);

    if let Some(&(_, rect)) = rects.iter().find(|(idx, _)| *idx == tab.active_pane_idx) {
        let pane = &mut tab.panes[tab.active_pane_idx];
        if pane.is_mouse_selecting {
            if let Some(anchor) = pane.selection_anchor {
                if let Some(coord) = get_char_coord_in_article_pane(pane, rect, col, row) {
                    pane.text_selection = Some(TextSelection {
                        start: anchor,
                        end: coord,
                    });
                    return true;
                }
            }
        }
    }
    false
}

pub fn handle_selection_up(app: &mut App) {
    let tab = app.active_tab_mut();
    let pane = &mut tab.panes[tab.active_pane_idx];
    if pane.is_mouse_selecting {
        pane.is_mouse_selecting = false;
        if let Some(selection) = pane.text_selection {
            let (start, end) = selection.normalized();
            if start != end {
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    let text = extract_selected_text(parsed_doc, &selection);
                    if !text.trim().is_empty() {
                        let count = text.chars().count();
                        if crate::clipboard::copy_to_clipboard(&text) {
                            app.set_status_message(format!("copied {} characters to clipboard", count));
                        }
                    }
                }
            }
        }
    }
}

pub fn extract_selected_text(
    doc: &crate::parser::ParsedDocument,
    selection: &TextSelection,
) -> String {
    let ((start_line, start_col), (end_line, end_col)) = selection.normalized();
    let mut lines_out = Vec::new();

    for line_idx in start_line..=end_line.min(doc.lines.len().saturating_sub(1)) {
        if let Some(line) = doc.lines.get(line_idx) {
            let full_line: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            let char_count = full_line.chars().count();
            let from = if line_idx == start_line {
                start_col.min(char_count)
            } else {
                0
            };
            let to = if line_idx == end_line {
                end_col.min(char_count)
            } else {
                char_count
            };

            if from < to {
                let slice: String = full_line.chars().skip(from).take(to - from).collect();
                lines_out.push(slice);
            } else if start_line != end_line {
                lines_out.push(String::new());
            }
        }
    }

    lines_out.join("\n")
}
