use crate::app::App;
use ratatui::{
    Frame,
    layout::Alignment,
    prelude::{Color, Style},
    widgets::{Block, Paragraph},
};

pub fn draw(f: &mut Frame, _app: &App) {
    let size = f.size();

    let block = Block::bordered()
        .border_style(Style::default().fg(Color::White))
        .title(" wikipedia tui ");

    let paragraph = Paragraph::new("press 'q' to exit")
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, size);
}
