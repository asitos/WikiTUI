use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    truncate_with_ellipsis(s, max_width, "...")
}

fn take_width_bytes(s: &str, max_width: usize) -> usize {
    let mut cur_w = 0;
    let mut byte_end = 0;
    for (idx, ch) in s.char_indices() {
        let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if cur_w + ch_w > max_width {
            break;
        }
        cur_w += ch_w;
        byte_end = idx + ch.len_utf8();
    }
    byte_end
}

pub fn truncate_with_ellipsis(s: &str, max_width: usize, ellipsis: &str) -> String {
    let total_width = UnicodeWidthStr::width(s);
    if total_width <= max_width {
        return s.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let ell_w = UnicodeWidthStr::width(ellipsis);
    if max_width <= ell_w {
        let byte_end = take_width_bytes(s, max_width);
        return s[..byte_end].to_string();
    }

    let byte_end = take_width_bytes(s, max_width - ell_w);
    format!("{}{}", &s[..byte_end], ellipsis)
}
