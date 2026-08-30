pub fn parse_duration_to_secs(dur_str: &str) -> Option<u64> {
    let clean: String = dur_str
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ':' || c == '.' {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect();
    let s = clean.trim();
    if s.is_empty() {
        return None;
    }

    for token in s.split_whitespace() {
        if token.contains(':') {
            let parts: Vec<&str> = token.split(':').collect();
            match parts.len() {
                2 => {
                    if let (Ok(m), Ok(sec)) =
                        (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>())
                    {
                        return Some(m * 60 + sec);
                    }
                }
                3 => {
                    if let (Ok(h), Ok(m), Ok(sec)) = (
                        parts[0].trim().parse::<u64>(),
                        parts[1].trim().parse::<u64>(),
                        parts[2].trim().parse::<u64>(),
                    ) {
                        return Some(h * 3600 + m * 60 + sec);
                    }
                }
                _ => {}
            }
        }
    }

    let mut total_secs = 0u64;
    let mut found_any = false;

    let words: Vec<&str> = s.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if let Ok(num) = w.parse::<u64>() {
            if i + 1 < words.len() {
                let unit = words[i + 1];
                if unit.starts_with("hour") || unit.starts_with("hr") || unit == "h" {
                    total_secs += num * 3600;
                    found_any = true;
                    i += 2;
                    continue;
                } else if unit.starts_with("minute") || unit.starts_with("min") || unit == "m" {
                    total_secs += num * 60;
                    found_any = true;
                    i += 2;
                    continue;
                } else if unit.starts_with("second") || unit.starts_with("sec") || unit == "s" {
                    total_secs += num;
                    found_any = true;
                    i += 2;
                    continue;
                }
            }
        } else if let Some(pos) = w.find(|c: char| c.is_alphabetic()) {
            let (digits, unit) = w.split_at(pos);
            if let Ok(num) = digits.parse::<u64>() {
                if unit.starts_with('h') {
                    total_secs += num * 3600;
                    found_any = true;
                } else if unit.starts_with('m') {
                    total_secs += num * 60;
                    found_any = true;
                } else if unit.starts_with('s') {
                    total_secs += num;
                    found_any = true;
                }
            }
        }
        i += 1;
    }

    if found_any && total_secs > 0 {
        Some(total_secs)
    } else {
        None
    }
}
