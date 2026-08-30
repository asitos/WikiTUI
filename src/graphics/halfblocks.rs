use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn render_halfblock_lines(
    pixels: &[RgbPixel],
    width: usize,
    height: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let rows = height.div_ceil(2);

    for row in 0..rows {
        let top_y = row * 2;
        let bot_y = top_y + 1;

        let mut spans = Vec::with_capacity(width);

        for x in 0..width {
            let top_pixel = if top_y < height {
                pixels.get(top_y * width + x).copied()
            } else {
                None
            };

            let bot_pixel = if bot_y < height {
                pixels.get(bot_y * width + x).copied()
            } else {
                None
            };

            let fg = top_pixel
                .map(|p| Color::Rgb(p.r, p.g, p.b))
                .unwrap_or(Color::Reset);
            let bg = bot_pixel
                .map(|p| Color::Rgb(p.r, p.g, p.b))
                .unwrap_or(Color::Reset);

            let style = Style::default().fg(fg).bg(bg);
            spans.push(Span::styled("▀", style));
        }

        lines.push(Line::from(spans));
    }

    lines
}
