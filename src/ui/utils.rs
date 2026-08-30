use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    truncate_with_ellipsis(s, max_width, "...")
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
        return s[..byte_end].to_string();
    }

    let target_w = max_width - ell_w;
    let mut cur_w = 0;
    let mut byte_end = 0;
    for (idx, ch) in s.char_indices() {
        let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if cur_w + ch_w > target_w {
            break;
        }
        cur_w += ch_w;
        byte_end = idx + ch.len_utf8();
    }
    format!("{}{}", &s[..byte_end], ellipsis)
}
