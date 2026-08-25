use crate::theme;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerType {
    Delete,     // red: deletion, copyright, serious issues
    Content,    // orange: citations, neutral point of view, accuracy
    Style,      // yellow: tone, formatting, cleanup
    Notice,     // blue: current events, general notices
    Move,       // violet: merge, split, move proposals
    Protection, // lime: page protection
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleBanner {
    pub banner_type: BannerType,
    pub message: String,
}

impl BannerType {
    pub fn color(&self) -> Color {
        match self {
            BannerType::Delete => theme::RED,
            BannerType::Content => theme::ORANGE,
            BannerType::Style => theme::YELLOW,
            BannerType::Notice => theme::BLUE,
            BannerType::Move => theme::VIOLET,
            BannerType::Protection => theme::LIME,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BannerType::Delete => "CRITICAL WARNING",
            BannerType::Content => "CONTENT WARNING",
            BannerType::Style => "STYLE WARNING",
            BannerType::Notice => "NOTICE",
            BannerType::Move => "PROPOSAL",
            BannerType::Protection => "PROTECTION NOTICE",
        }
    }
}

pub fn classify_ambox_class(class_attr: &str) -> Option<BannerType> {
    let lower = class_attr.to_lowercase();

    // Must be a real ambox (Article Message Box)
    let is_ambox = class_attr
        .split_whitespace()
        .any(|cls| cls == "ambox" || cls.starts_with("ambox-"));

    if !is_ambox {
        return None;
    }

    if lower.contains("ambox-delete")
        || lower.contains("ambox-speedy")
        || lower.contains("ambox-serious")
        || lower.contains("ambox-deletion")
    {
        return Some(BannerType::Delete);
    }

    if lower.contains("ambox-content")
        || lower.contains("ambox-unreferenced")
        || lower.contains("ambox-dispute")
        || lower.contains("ambox-neutral")
        || lower.contains("ambox-accuracy")
    {
        return Some(BannerType::Content);
    }

    if lower.contains("ambox-style")
        || lower.contains("ambox-tone")
        || lower.contains("ambox-cleanup")
        || lower.contains("ambox-lead_section")
        || lower.contains("ambox-format")
    {
        return Some(BannerType::Style);
    }

    if lower.contains("ambox-move")
        || lower.contains("ambox-merge")
        || lower.contains("ambox-split")
        || lower.contains("ambox-translate")
    {
        return Some(BannerType::Move);
    }

    if lower.contains("ambox-protection")
        || lower.contains("ambox-protected")
        || lower.contains("ambox-semi-protected")
    {
        return Some(BannerType::Protection);
    }

    if lower.contains("ambox-notice")
        || lower.contains("ambox-current")
        || lower.contains("ambox-event")
        || lower.contains("ambox-talk")
    {
        return Some(BannerType::Notice);
    }

    Some(BannerType::Style)
}

pub fn clean_ambox_text(raw_text: &str) -> String {
    let mut clean_text = String::with_capacity(raw_text.len());

    for word in raw_text.split_whitespace() {
        if matches!(word, "." | "," | ":" | ";" | "!" | "?") {
            if clean_text.ends_with(' ') {
                clean_text.pop();
            }
            clean_text.push_str(word);
        } else {
            if !clean_text.is_empty() && !clean_text.ends_with(' ') {
                clean_text.push(' ');
            }
            clean_text.push_str(word);
        }
    }

    if let Some(idx) = clean_text.find("( Learn how") {
        clean_text.truncate(idx);
    }
    if let Some(idx) = clean_text.find("(Learn how") {
        clean_text.truncate(idx);
    }
    if let Some(open_paren) = clean_text.rfind('(') {
        let trailing = &clean_text[open_paren..];
        if trailing.contains("202") || trailing.contains("201") || trailing.contains("200") {
            clean_text.truncate(open_paren);
        }
    }

    clean_text.trim().to_string()
}
