use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

struct CachedKittyPayload {
    b64: String,
    width: u32,
    height: u32,
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
        out.push(if chunk.len() > 1 {
            BASE64_ALPHABET[(triple >> 6) & 0x3F] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64_ALPHABET[triple & 0x3F] as char
        } else {
            '='
        });
    }
    out
}

pub fn clear_all_kitty_images<W: Write>(writer: &mut W) -> io::Result<()> {
    write!(writer, "\x1b_Ga=d\x1b\\")?;
    writer.flush()
}

pub struct KittyImageArgs<'a> {
    pub path: &'a Path,
    pub screen_x: u16,
    pub screen_y: u16,
    pub cols: u16,
    pub rows: u16,
    pub crop_top_lines: u16,
    pub crop_bot_lines: u16,
}

pub fn render_kitty_image_from_path<W: Write>(
    writer: &mut W,
    args: KittyImageArgs<'_>,
) -> io::Result<()> {
    let mut cache_guard = KITTY_PAYLOAD_CACHE.lock().unwrap();
    let cache = cache_guard.get_or_insert_with(HashMap::new);

    if let Some(payload) = cache.get(args.path) {
        write!(writer, "\x1b[{};{}H", args.screen_y + 1, args.screen_x + 1)?;
        render_kitty_rgba_chunked(writer, &payload.b64, payload.width, payload.height, &args)?;
        return Ok(());
    }

    let Ok(image_bytes) = std::fs::read(args.path) else {
        return Ok(());
    };

    if let Ok(img) = image::load_from_memory(&image_bytes) {
        let (w, h) = (img.width(), img.height());
        let mut rgba = img.to_rgba8();
        for pixel in rgba.pixels_mut() {
            let a = pixel[3] as u32;
            if a < 255 {
                let inv_a = 255 - a;
                pixel[0] = ((pixel[0] as u32 * a + 255 * inv_a) / 255) as u8;
                pixel[1] = ((pixel[1] as u32 * a + 255 * inv_a) / 255) as u8;
                pixel[2] = ((pixel[2] as u32 * a + 255 * inv_a) / 255) as u8;
                pixel[3] = 255;
            }
        }
        let b64 = base64_encode(&rgba);
        write!(writer, "\x1b[{};{}H", args.screen_y + 1, args.screen_x + 1)?;
        render_kitty_rgba_chunked(writer, &b64, w, h, &args)?;
        cache.insert(
            args.path.to_path_buf(),
            CachedKittyPayload {
                b64,
                width: w,
                height: h,
            },
        );
    }

    Ok(())
}

pub fn render_kitty_rgba_chunked<W: Write>(
    writer: &mut W,
    rgba_base64: &str,
    img_w: u32,
    img_h: u32,
    args: &KittyImageArgs<'_>,
) -> io::Result<()> {
    let total_lines = (args.rows + args.crop_top_lines + args.crop_bot_lines) as u32;
    
    let mut crop_str = String::new();
    if total_lines > 0 && (args.crop_top_lines > 0 || args.crop_bot_lines > 0) {
        let y_pixel = (args.crop_top_lines as u32 * img_h) / total_lines;
        let h_pixel = (args.rows as u32 * img_h) / total_lines;
        crop_str = format!(",y={},h={}", y_pixel, h_pixel);
    }

    let mut chunks = rgba_base64.as_bytes().chunks(4096).peekable();
    if let Some(first) = chunks.next() {
        let first_str = std::str::from_utf8(first).unwrap_or_default();
        let more = if chunks.peek().is_some() { 1 } else { 0 };
        write!(
            writer,
            "\x1b_Ga=T,f=32,s={},v={},c={},r={},m={}{};{}\x1b\\",
            img_w, img_h, args.cols, args.rows, more, crop_str, first_str
        )?;
        while let Some(chunk) = chunks.next() {
            let chunk_str = std::str::from_utf8(chunk).unwrap_or_default();
            let more = if chunks.peek().is_some() { 1 } else { 0 };
            write!(writer, "\x1b_Gm={};{}\x1b\\", more, chunk_str)?;
        }
    }
    writer.flush()
}
