use crate::app::{App, Pane};
use crate::parser::ParsedDocument;
use crate::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, block::Title},
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_help_modal(f: &mut Frame, size: Rect) {
    let area = centered_rect(70, 85, size);
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            " navigation",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   j/k            scroll down / up"),
        Line::from("   f/b            scroll page down / up"),
        Line::from("   g/G            jump to top / bottom"),
        Line::from("   ]/[            jump to next / prev section heading"),
        Line::from("   o              toggle table of contents"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " links & selection",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   tab/backtab    focus next / prev link"),
        Line::from("   enter          open link in current pane"),
        Line::from("   t              open link in new tab"),
        Line::from("   s/v            open link in horizontal / vertical split"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " panes & tabs",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   ctrl-w s/v     split active pane horizontally / vertically"),
        Line::from("   ctrl-h/j/k/l   navigate focus between split panes"),
        Line::from("   alt-c          close active pane"),
        Line::from("   ctrl-t         create new tab"),
        Line::from("   alt-h/l        switch to prev / next tab"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " search",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   ctrl-s         search wikipedia (opens new tab)"),
        Line::from("   i              edit search query in current tab"),
        Line::from("   /              in-page text search"),
        Line::from("   n/N            jump to next / prev search match"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " general",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   z              toggle zen mode"),
        Line::from("   ?              toggle this help popup"),
        Line::from("   q              quit wiki-tui"),
    ];

    let help_block = Block::bordered()
        .border_style(Style::default().fg(theme::PINK))
        .title(Title::from(" keybindings ").alignment(Alignment::Center))
        .title(
            Title::from(Span::styled(
                " esc to close ",
                Style::default().fg(theme::GREY).italic(),
            ))
            .position(ratatui::widgets::block::Position::Bottom)
            .alignment(Alignment::Right),
        );

    let help_paragraph = Paragraph::new(help_text).block(help_block);
    f.render_widget(help_paragraph, area);
}

pub fn render_search_modal(f: &mut Frame, app: &App, size: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Length(3), // 1 character tall
            Constraint::Min(0),
        ])
        .split(size);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(28),
            Constraint::Percentage(36),
        ])
        .split(popup_layout[1])[1];

    f.render_widget(Clear, area);

    let search_block = Block::bordered()
        .border_style(Style::default().fg(theme::BEIGE))
        .title(Title::from(" search wikipedia ").alignment(Alignment::Left));

    let input_text = Line::from(vec![
        Span::styled(" > ", Style::default().fg(theme::BEIGE).bold()),
        Span::styled(&app.search_input, Style::default().fg(theme::FG).bold()),
        Span::styled("_", Style::default().fg(theme::BEIGE).bold()),
    ]);

    let search_paragraph = Paragraph::new(input_text).block(search_block);
    f.render_widget(search_paragraph, area);
}

pub fn render_toc_modal(
    f: &mut Frame,
    pane: &Pane,
    parsed_doc: &ParsedDocument,
    container_rect: Rect,
) {
    let toc_area = centered_rect(60, 60, container_rect);
    f.render_widget(Clear, toc_area);

    let toc_block = Block::bordered()
        .border_style(Style::default().fg(theme::LIME))
        .title(" contents ")
        .title(
            Title::from(" enter: jump | o: close ")
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Right),
        );

    let current_scroll = pane.scroll_offset;
    let active_heading_idx = parsed_doc
        .headings
        .iter()
        .rposition(|h| h.line_idx <= current_scroll)
        .unwrap_or(0);

    let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

    let mut toc_lines = Vec::new();
    for (idx, h) in parsed_doc.headings.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let indent_len = ((h.level.saturating_sub(1)) * 2) as usize;
        let indent = " ".repeat(indent_len);
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default().fg(theme::LIME).bold()
        } else {
            Style::default().fg(theme::FG)
        };

        let avail_w = (toc_area.width as usize).saturating_sub(6 + indent_len);
        let truncated_title = if h.title.len() > avail_w && avail_w > 3 {
            format!("{}...", &h.title[..avail_w.saturating_sub(3)])
        } else {
            h.title.clone()
        };

        toc_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(indent),
            Span::styled(truncated_title, style),
        ]));
    }

    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
    let toc_scroll = selected_idx.saturating_sub(visible_rows / 2);

    let toc_p = Paragraph::new(toc_lines)
        .block(toc_block)
        .scroll((toc_scroll as u16, 0));
    f.render_widget(toc_p, toc_area);
}
