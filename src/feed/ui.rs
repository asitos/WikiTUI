use crate::feed::FeedState;
use crate::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_feed_view(f: &mut Frame, feed: &FeedState, area: Rect) {
    f.render_widget(Clear, area);

    let main_block = Block::bordered()
        .border_style(Style::default().fg(theme::VIOLET))
        .title(Title::from(" wikipedia feed ").alignment(Alignment::Center));

    f.render_widget(main_block.clone(), area);
    let inner_area = main_block.inner(area);

    if feed.items.is_empty() {
        let vertical_offset = (inner_area.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            " fetching articles for your feed... ",
            Style::default().fg(theme::YELLOW).bold(),
        )));

        let loading_p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(loading_p, inner_area);
        return;
    }

    let active_idx = feed.active_idx;
    let item = &feed.items[active_idx];

    let card_area = centered_rect(80, 85, inner_area);
    f.render_widget(Clear, card_area);

    let card_border_color = if item.is_liked {
        theme::LIME
    } else {
        theme::PINK
    };

    let like_badge = if item.is_liked { " liked " } else { "" };
    let like_style = if item.is_liked {
        Style::default().fg(theme::LIME).bold()
    } else {
        Style::default().fg(theme::GREY)
    };

    let card_block = Block::bordered()
        .border_style(Style::default().fg(card_border_color))
        .title(Title::from(format!(" {} ", item.title.to_lowercase())).alignment(Alignment::Center))
        .title(
            Title::from(format!(" post {} of {} ", active_idx + 1, feed.items.len()))
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Left),
        )
        .title(
            Title::from(Span::styled(like_badge, like_style))
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Right),
        );

    let mut card_lines = Vec::new();
    card_lines.push(Line::from(""));

    if let Some(short_desc) = &item.short_description {
        if !short_desc.is_empty() {
            card_lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(short_desc, Style::default().fg(theme::GREY).italic()),
            ]));
            card_lines.push(Line::from(""));
        }
    }

    if !item.snippet.is_empty() {
        card_lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(&item.snippet, Style::default().fg(theme::FG)),
        ]));
    } else {
        card_lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                "press enter to read full article...",
                Style::default().fg(theme::GREY).italic(),
            ),
        ]));
    }

    card_lines.push(Line::from(""));
    if !item.categories.is_empty() {
        let tags: Vec<String> = item.categories.iter().map(|c| format!("#{}", c)).collect();
        card_lines.push(Line::from(vec![
            Span::styled("   categories: ", Style::default().fg(theme::GREY)),
            Span::styled(tags.join(" "), Style::default().fg(theme::VIOLET)),
        ]));
    }

    let card_p = Paragraph::new(card_lines)
        .block(card_block)
        .wrap(Wrap { trim: true });
    f.render_widget(card_p, card_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
