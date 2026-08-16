use super::utils::centered_rect;
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph},
    Frame,
};

#[rustfmt::skip]
pub fn render_help_modal(f: &mut Frame, size: Rect) {
    let area = centered_rect(80, 90, size);
    f.render_widget(Clear, area);

    let is_macos = cfg!(target_os = "macos");

    let mut help_text = vec![
        Line::from(vec![Span::styled(
            " navigation",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   j/k                 scroll down / up"),
        Line::from("   f/b                 scroll page down / up"),
        Line::from("   g/G                 jump to top / bottom"),
        Line::from("   H/L                 go backward / forward in article history"),
        Line::from("   ]/[                 jump to next / prev section heading"),
        Line::from("   o                   toggle table of contents"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " links & selection",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
        Line::from("   tab/backtab         focus next / prev link"),
        Line::from("   enter               open link in current pane"),
        Line::from("   t                   open link in new tab"),
        Line::from("   s/v                 open link in horizontal / vertical split"),
        Line::from("   y                   copy focused link to clipboard"),
        Line::from("   Y                   copy current article URL to clipboard"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " panes & tabs",
            Style::default().fg(theme::VIOLET).bold(),
        )]),
    ];

    let opt_label = if is_macos { "opt" } else { "alt" };
    help_text.push(Line::from("   ctrl-w s/v          split active pane horizontally / vertically"));
    help_text.push(Line::from("   ctrl-h/j/k/l        navigate focus between split panes"));
    help_text.push(Line::from(format!("   x / {}-c           close active pane", opt_label)));
    help_text.push(Line::from(format!("   {}-shift-c         reopen last closed tab/pane", opt_label)));
    help_text.push(Line::from(format!("   {}-t               create new tab", opt_label)));
    help_text.push(Line::from(format!("   {}-h/l             switch to prev / next tab", opt_label)));
    help_text.push(Line::from(format!("   {}-0..9            switch to tab 1-10", opt_label)));

    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " search",
        Style::default().fg(theme::VIOLET).bold(),
    )]));

    help_text.push(Line::from("   ctrl-s              search wikipedia (opens new tab)"));

    help_text.push(Line::from("   i                   edit search query in current tab"));
    help_text.push(Line::from("   r                   open random wikipedia article"));
    help_text.push(Line::from("   /                   in-page text search"));
    help_text.push(Line::from("   n/N                 jump to next / prev search match"));
    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " custom lists",
        Style::default().fg(theme::VIOLET).bold(),
    )]));
    help_text.push(Line::from("   m                   save active article to custom list"));
    help_text.push(Line::from("   M                   open saved custom lists & articles viewer"));
    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        " general",
        Style::default().fg(theme::VIOLET).bold(),
    )]));
    help_text.push(Line::from("   z                   toggle zen mode"));
    help_text.push(Line::from("   F                   toggle wikipedia feed mode"));
    help_text.push(Line::from("   S                   restore previous session"));
    help_text.push(Line::from("   ?                   toggle this help popup"));
    help_text.push(Line::from("   q                   quit wikid"));

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
