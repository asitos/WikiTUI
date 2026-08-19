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
    r#"             .--.     .     .--.\--___-`'.   "#,
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

pub fn render_launch_screen(f: &mut Frame, _app: &App, rect: Rect, block: Block) {
    let inner_width = (rect.width as usize).saturating_sub(4);
    let inner_height = (rect.height as usize).saturating_sub(2);

    let left_pad = (inner_width.saturating_sub(LOGO_WIDTH)) / 2;
    let pad_str = " ".repeat(left_pad);

    let mut lines = Vec::new();
    let total_logo_height = LOGO.len() + 2;
    let v_pad = inner_height.saturating_sub(total_logo_height) / 2;

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
    let subtitle = "wikipedia reader for the terminal";
    let sub_pad = (inner_width.saturating_sub(subtitle.len())) / 2;
    lines.push(Line::from(vec![
        Span::raw(" ".repeat(sub_pad)),
        Span::styled(subtitle, Style::default().fg(theme::GREY).italic()),
    ]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, rect);
}
