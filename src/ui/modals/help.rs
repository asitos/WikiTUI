use super::utils::centered_rect;
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph},
    Frame,
};

#[rustfmt::skip]
pub fn render_help_modal(f: &mut Frame, size: Rect) {
    let area = centered_rect(70, 80, size);
    f.render_widget(Clear, area);

    let is_macos = cfg!(target_os = "macos");
    let opt_label = if is_macos { "opt" } else { "alt" };

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

    let inner = help_block.inner(area);
    f.render_widget(help_block, area);

    if inner.width < 70 {
        let help_text = vec![
            Line::from(vec![Span::styled(
                " navigation",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   j/k                 scroll down / up"),
            Line::from("   f/b                 scroll page down / up"),
            Line::from("   g/G                 jump to top / bottom"),
            Line::from("   ctrl-o/i            jump backward / forward in jumps"),
            Line::from("   H/L                 history backward / forward"),
            Line::from("   ]/[                 jump to next / prev heading"),
            Line::from("   o                   toggle table of contents"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " links & selection",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   tab/backtab         focus next / prev link"),
            Line::from("   enter               open link in current pane"),
            Line::from("   t                   open link in new tab"),
            Line::from("   s/v                 open in horizontal / vertical split"),
            Line::from("   y                   copy focused link to clipboard"),
            Line::from("   Y                   copy article URL to clipboard"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " panes & tabs",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   ctrl-w s/v          split pane horiz / vert"),
            Line::from("   ctrl-h/j/k/l        navigate focus between panes"),
            Line::from(format!("   x / {}-c           close active pane", opt_label)),
            Line::from(format!("   {}-shift-c         reopen last closed tab/pane", opt_label)),
            Line::from(format!("   {}-t               create new tab", opt_label)),
            Line::from(format!("   {}-h/l             switch to prev / next tab", opt_label)),
            Line::from(format!("   {}-0..9            switch to tab 1-10", opt_label)),
            Line::from(""),
            Line::from(vec![Span::styled(
                " search",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   ctrl-s              search wikipedia"),
            Line::from("   0..9                open search result directly"),
            Line::from("   i                   edit search query"),
            Line::from("   r                   open random article"),
            Line::from("   /                   in-page text search"),
            Line::from("   n/N                 jump to next / prev match"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " custom lists",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   m                   save to custom list"),
            Line::from("   M                   open saved lists viewer"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " general",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   z                   toggle zen mode"),
            Line::from("   F                   toggle feed mode"),
            Line::from("   S                   restore session"),
            Line::from("   ?                   toggle help"),
            Line::from("   q                   quit wikid"),
        ];
        f.render_widget(Paragraph::new(help_text), inner);
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(2),
                Constraint::Percentage(50),
            ])
            .split(inner);

        let left_lines = vec![
            Line::from(vec![Span::styled(
                " navigation",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   j/k                 scroll down / up"),
            Line::from("   f/b                 scroll page down / up"),
            Line::from("   g/G                 jump to top / bottom"),
            Line::from("   ctrl-o/i            jump backward / forward in jumps"),
            Line::from("   H/L                 go backward / forward in history"),
            Line::from("   ]/[                 jump to next / prev heading"),
            Line::from("   o                   toggle table of contents"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " links & selection",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   tab/backtab         focus next / prev link"),
            Line::from("   enter               open link in current pane"),
            Line::from("   t                   open link in new tab"),
            Line::from("   s/v                 open in horiz / vert split"),
            Line::from("   y                   copy link to clipboard"),
            Line::from("   Y                   copy article URL to clipboard"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " general",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   z                   toggle zen mode"),
            Line::from("   F                   toggle wikipedia feed mode"),
            Line::from("   S                   restore previous session"),
            Line::from("   ?                   toggle this help popup"),
            Line::from("   q                   quit wikid"),
        ];

        let right_lines = vec![
            Line::from(vec![Span::styled(
                " panes & tabs",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   ctrl-w s/v          split pane horiz / vert"),
            Line::from("   ctrl-h/j/k/l        navigate focus between panes"),
            Line::from(format!("   x / {}-c           close active pane", opt_label)),
            Line::from(format!("   {}-shift-c         reopen last closed tab/pane", opt_label)),
            Line::from(format!("   {}-t               create new tab", opt_label)),
            Line::from(format!("   {}-h/l             switch to prev / next tab", opt_label)),
            Line::from(format!("   {}-0..9            switch to tab 1-10", opt_label)),
            Line::from(""),
            Line::from(vec![Span::styled(
                " search",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   ctrl-s              search wikipedia (new tab)"),
            Line::from("   0..9                open search result directly"),
            Line::from("   i                   edit search query in tab"),
            Line::from("   r                   open random article"),
            Line::from("   /                   in-page text search"),
            Line::from("   n/N                 jump to next / prev match"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " custom lists",
                Style::default().fg(theme::VIOLET).bold(),
            )]),
            Line::from("   m                   save article to custom list"),
            Line::from("   M                   open saved lists viewer"),
        ];

        f.render_widget(Paragraph::new(left_lines), cols[0]);
        f.render_widget(Paragraph::new(right_lines), cols[2]);
    }
}
