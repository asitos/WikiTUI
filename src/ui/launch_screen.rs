use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub const LOGO_WIDTH: usize = 53;
pub const LOGO: &[&str] = &[
    r#"                                             "#,
    r#"             .--.     .     .--.\  ___ `'.   "#,
    r#"      _     _|__|   .'|     |__| ' |--.\  \  "#,
    r#"/\    \\   //.--. .'  |     .--. | |    \  ' "#,
    r#"`\\  //\\ // |  |<    |     |  | | |     |  '"#,
    r#"  \`//  \'/  |  | |   | ____|  | | |     |  |"#,
    r#"   \|   |/   |  | |   | \ .'|  | | |     ' .'"#,
    r#"    '        |  | |   |/  . |  | | |___.' /' "#,
    r#"             |__| |    /\  \|__|/_______.'/  "#,
    r#"                  |   |  \  \   \_______|/   "#,
    r#"                  '    \  \  \               "#,
    r#"                 '------'  '---'             "#,
];

pub const QUOTES: &[&str] = &[
    "wikipedia reader for the terminal",
    "it's wicked",
    "just one more before bed",
    "genuine autism",
    "press r",
];

pub fn render_launch_screen(f: &mut Frame, app: &App, rect: Rect, block: Block) {
    let inner_width = (rect.width as usize).saturating_sub(4);
    let inner_height = (rect.height as usize).saturating_sub(2);

    let left_pad = (inner_width.saturating_sub(LOGO_WIDTH)) / 2;
    let pad_str = " ".repeat(left_pad);

    let recent_articles = app.get_continue_reading_articles();
    let show_recent = !recent_articles.is_empty() && inner_height >= (LOGO.len() + 8);
    let displayed_count = if show_recent {
        recent_articles.len().min(7)
    } else {
        0
    };

    let mut lines = Vec::new();
    let total_content_height = LOGO.len() + 4 + if show_recent { displayed_count + 2 } else { 0 };
    let v_pad = inner_height.saturating_sub(total_content_height) / 2;

    for _ in 0..v_pad {
        lines.push(Line::from(""));
    }

    for &line in LOGO {
        lines.push(Line::from(vec![
            Span::raw(pad_str.clone()),
            Span::styled(line, Style::default().fg(theme::PINK).bold()),
        ]));
    }

    lines.push(Line::from(""));
    let quote = QUOTES[app.launch_quote_idx % QUOTES.len()];
    let quote_len = quote.chars().count();
    if inner_width >= quote_len {
        let q_pad = (inner_width.saturating_sub(quote_len)) / 2;
        lines.push(Line::from(vec![
            Span::raw(" ".repeat(q_pad)),
            Span::styled(quote, Style::default().fg(theme::GREY).italic()),
        ]));
    } else {
        let default_sub = "wikipedia reader for the terminal";
        let sub_pad = (inner_width.saturating_sub(default_sub.len())) / 2;
        lines.push(Line::from(vec![
            Span::raw(" ".repeat(sub_pad)),
            Span::styled(default_sub, Style::default().fg(theme::GREY).italic()),
        ]));
    }

    lines.push(Line::from(""));
    let stats_spans = if inner_width >= 75 {
        vec![
            Span::styled(
                format!(
                    "󰈙 {} articles",
                    crate::api::stats::format_metric(app.wiki_stats.articles)
                ),
                Style::default().fg(theme::LIME),
            ),
            Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
            Span::styled(
                format!(
                    "󰑐 {} edits",
                    crate::api::stats::format_metric(app.wiki_stats.edits)
                ),
                Style::default().fg(theme::TEAL),
            ),
            Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
            Span::styled(
                format!(
                    "󰒓 {} active editors",
                    crate::api::stats::format_metric(app.wiki_stats.activeusers)
                ),
                Style::default().fg(theme::YELLOW),
            ),
            Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
            Span::styled(
                format!(
                    "󰠱 {} pages",
                    crate::api::stats::format_metric(app.wiki_stats.pages)
                ),
                Style::default().fg(theme::VIOLET),
            ),
        ]
    } else {
        vec![
            Span::styled(
                format!(
                    "󰈙 {} articles",
                    crate::api::stats::format_metric(app.wiki_stats.articles)
                ),
                Style::default().fg(theme::LIME),
            ),
            Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
            Span::styled(
                format!(
                    "󰒓 {} active editors",
                    crate::api::stats::format_metric(app.wiki_stats.activeusers)
                ),
                Style::default().fg(theme::YELLOW),
            ),
        ]
    };

    let stats_width: usize = stats_spans
        .iter()
        .map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref()))
        .sum();
    let stats_pad = (inner_width.saturating_sub(stats_width)) / 2;
    let mut stats_line = vec![Span::raw(" ".repeat(stats_pad))];
    stats_line.extend(stats_spans);
    lines.push(Line::from(stats_line));

    if show_recent {
        lines.push(Line::from(""));
        let header = "── continue reading ──";
        let h_pad = (inner_width.saturating_sub(header.len())) / 2;
        lines.push(Line::from(vec![
            Span::raw(" ".repeat(h_pad)),
            Span::styled(header, Style::default().fg(theme::GREY)),
        ]));

        let items: Vec<(String, String)> = recent_articles
            .iter()
            .take(displayed_count)
            .enumerate()
            .map(|(idx, title)| (format!("{}. ", idx + 1), title.clone()))
            .collect();

        let max_item_width = items
            .iter()
            .map(|(p, t)| p.len() + unicode_width::UnicodeWidthStr::width(t.as_str()))
            .max()
            .unwrap_or(0);

        let block_pad = (inner_width.saturating_sub(max_item_width)) / 2;

        for (num_prefix, title) in items {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(block_pad)),
                Span::styled(num_prefix, Style::default().fg(theme::GREY)),
                Span::styled(title, Style::default().fg(theme::FG)),
            ]));
        }
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, rect);
}
