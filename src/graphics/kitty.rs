use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

enum CachedKittyPayload {
    Png(String),
    Rgba {
        b64: String,
        width: u32,
        height: u32,
    },
}

static KITTY_PAYLOAD_CACHE: Mutex<Option<HashMap<PathBuf, CachedKittyPayload>>> = Mutex::new(None);

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(BASE64_ALPHABET[(triple >> 18) & 0x3F] as char);
        out.push(BASE64_ALPHABET[(triple >> 12) & 0x3F] as char);

        if chunk.len() > 1 {
            out.push(BASE64_ALPHABET[(triple >> 6) & 0x3F] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(BASE64_ALPHABET[triple & 0x3F] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub fn clear_all_kitty_images<W: Write>(writer: &mut W) -> io::Result<()> {
    write!(writer, "\x1b_Ga=d\x1b\\")?;
    writer.flush()
}

pub fn render_kitty_image_from_path<W: Write>(
    writer: &mut W,
    path: &Path,
    screen_x: u16,
    screen_y: u16,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let mut cache_guard = KITTY_PAYLOAD_CACHE.lock().unwrap();
    let cache = cache_guard.get_or_insert_with(HashMap::new);

    if let Some(payload) = cache.get(path) {
        write!(writer, "\x1b[{};{}H", screen_y + 1, screen_x + 1)?;
        match payload {
            CachedKittyPayload::Png(b64) => render_kitty_png_chunked(writer, b64, cols, rows)?,
            CachedKittyPayload::Rgba { b64, width, height } => {
                render_kitty_rgba_chunked(writer, b64, *width, *height, cols, rows)?
            }
        }
        return Ok(());
    }

    let Ok(image_bytes) = std::fs::read(path) else {
        return Ok(());
    };

    if image_bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        let b64 = base64_encode(&image_bytes);
        write!(writer, "\x1b[{};{}H", screen_y + 1, screen_x + 1)?;
        render_kitty_png_chunked(writer, &b64, cols, rows)?;
        cache.insert(path.to_path_buf(), CachedKittyPayload::Png(b64));
    } else if let Ok(img) = image::load_from_memory(&image_bytes) {
        let (w, h) = (img.width(), img.height());
        let rgba = img.to_rgba8();
        let b64 = base64_encode(&rgba);
        write!(writer, "\x1b[{};{}H", screen_y + 1, screen_x + 1)?;
        render_kitty_rgba_chunked(writer, &b64, w, h, cols, rows)?;
        cache.insert(
            path.to_path_buf(),
            CachedKittyPayload::Rgba {
                b64,
                width: w,
                height: h,
            },
        );
    }

    Ok(())
}

pub fn render_kitty_png_chunked<W: Write>(
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

pub fn render_kitty_rgba_chunked<W: Write>(
    writer: &mut W,
    rgba_base64: &str,
    img_w: u32,
    img_h: u32,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let chunk_size = 4096;
    let bytes = rgba_base64.as_bytes();
    let total = bytes.len();
    let mut offset = 0;

    while offset < total {
        let end = (offset + chunk_size).min(total);
        let chunk = &rgba_base64[offset..end];
        let more = if end < total { 1 } else { 0 };

        if offset == 0 {
            write!(
                writer,
                "\x1b_Ga=T,f=32,s={},v={},c={},r={},m={};{}\x1b\\",
                img_w, img_h, cols, rows, more, chunk
            )?;
        } else {
            write!(writer, "\x1b_Gm={};{}\x1b\\", more, chunk)?;
        }

        offset = end;
    }

    writer.flush()
}
