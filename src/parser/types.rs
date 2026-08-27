use ratatui::style::Style;
use ratatui::text::Line;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub title: String,
    pub level: u8,
    pub line_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub title: String,
    pub text: String,
    pub span_indices: Vec<(usize, usize)>,
}

impl Link {
    pub fn is_external(&self) -> bool {
        self.title.starts_with("http://")
            || self.title.starts_with("https://")
            || self.title.starts_with("//")
    }

    pub fn is_citation(&self) -> bool {
        self.title.starts_with("#cite_note")
            || self.title.starts_with("#cite_ref")
            || self.title.starts_with("cite_note")
            || self.title.starts_with("cite_ref")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTrack {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpokenAudio {
    pub title: String,
    pub duration: Option<String>,
    pub tracks: Vec<AudioTrack>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    pub lines: Vec<Line<'static>>,
    pub plain_text_lower: Vec<String>,
    pub links: Vec<Link>,
    pub headings: Vec<Heading>,
    pub reference_targets: HashMap<String, usize>,
    pub spoken_audio: Option<SpokenAudio>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct StyledToken {
    pub text: String,
    pub style: Style,
    pub link_target: Option<String>,
}
