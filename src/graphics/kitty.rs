use std::io::{self, Write};

pub fn clear_all_kitty_images<W: Write>(writer: &mut W) -> io::Result<()> {
    write!(writer, "\x1b_Ga=d\x1b\\")?;
    writer.flush()
}

pub fn render_kitty_image_chunked<W: Write>(
    writer: &mut W,
    png_base64: &str,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let chunk_size = 4096;
    let bytes = png_base64.as_bytes();
    let total = bytes.len();
    let mut offset = 0;

    while offset < total {
        let end = (offset + chunk_size).min(total);
        let chunk = &png_base64[offset..end];
        let more = if end < total { 1 } else { 0 };

        if offset == 0 {
            write!(
                writer,
                "\x1b_Ga=T,f=100,c={},r={},m={};{}\x1b\\",
                cols, rows, more, chunk
            )?;
        } else {
            write!(writer, "\x1b_Gm={};{}\x1b\\", more, chunk)?;
        }

        offset = end;
    }

    writer.flush()
}
