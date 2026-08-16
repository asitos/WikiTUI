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

pub(crate) fn extract_title_from_href(href: &str) -> Option<String> {
    let trimmed = href.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(anchor) = trimmed.strip_prefix('#') {
        if anchor.starts_with("cite_note") || anchor.starts_with("cite_ref") {
            return Some(trimmed.to_string());
        }
        return None;
    }

    let wiki_path = if let Some(p) = trimmed.strip_prefix("/wiki/") {
        Some(p)
    } else if let Some(p) = trimmed.strip_prefix("./") {
        Some(p)
    } else if let Some(p) = trimmed.strip_prefix("https://en.wikipedia.org/wiki/") {
        Some(p)
    } else if let Some(p) = trimmed.strip_prefix("http://en.wikipedia.org/wiki/") {
        Some(p)
    } else if let Some(p) = trimmed.strip_prefix("//en.wikipedia.org/wiki/") {
        Some(p)
    } else {
        trimmed.find("/w/index.php?title=").map(|idx| &trimmed[idx + 19..])
    };

    if let Some(path) = wiki_path {
        let raw_title = path.split('#').next().unwrap_or(path);
        let raw_title = raw_title.split('&').next().unwrap_or(raw_title);

        if let Some(colon_idx) = raw_title.find(':') {
            let prefix = &raw_title[..colon_idx];
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
        let decoded = url_decode(raw_title).replace('_', " ");
        let cleaned = decoded.trim().to_string();
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.starts_with("//") {
        let full_url = if trimmed.starts_with("//") {
            format!("https:{}", trimmed)
        } else {
            trimmed.to_string()
        };
        return Some(full_url);
    }

    None
}
