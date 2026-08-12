pub fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut i = 0;
    let s_bytes = s.as_bytes();
    while i < s_bytes.len() {
        if s_bytes[i] == b'%' && i + 2 < s_bytes.len() {
            if let Ok(hex_str) = std::str::from_utf8(&s_bytes[i + 1..i + 3]) {
                if let Ok(b) = u8::from_str_radix(hex_str, 16) {
                    bytes.push(b);
                    i += 3;
                    continue;
                }
            }
        }
        if s_bytes[i] == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(s_bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&bytes).to_string()
}

// extract title from wiki links
pub(crate) fn extract_title_from_href(href: &str) -> Option<String> {
    if let Some(path) = href.strip_prefix("/wiki/") {
        if let Some(colon_idx) = path.find(':') {
            let prefix = &path[..colon_idx];
            if matches!(
                prefix,
                "Special"
                    | "File"
                    | "Category"
                    | "Help"
                    | "Wikipedia"
                    | "Template"
                    | "User"
                    | "Talk"
                    | "Portal"
                    | "Draft"
                    | "MediaWiki"
                    | "Media"
            ) || prefix.ends_with("_talk")
            {
                return None;
            }
        }
        let raw_title = path.split('#').next().unwrap_or(path);
        let decoded = url_decode(raw_title).replace('_', " ");
        Some(decoded)
    } else {
        None
    }
}
