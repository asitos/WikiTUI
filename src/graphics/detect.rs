use super::protocol::GraphicsProtocol;
use std::env;

pub fn detect_terminal_protocol() -> GraphicsProtocol {
    if env::var("KITTY_WINDOW_ID").is_ok()
        || env::var("GHOSTTY_RESOURCES_DIR").is_ok()
        || env::var("WEZTERM_EXECUTABLE").is_ok()
        || env::var("TERM_PROGRAM")
            .map(|tp| {
                let lower = tp.to_lowercase();
                lower.contains("kitty") || lower.contains("ghostty") || lower.contains("wezterm")
            })
            .unwrap_or(false)
        || env::var("TERM")
            .map(|t| {
                let lower = t.to_lowercase();
                lower.contains("kitty") || lower.contains("ghostty") || lower.contains("wezterm")
            })
            .unwrap_or(false)
    {
        GraphicsProtocol::Kitty
    } else {
        GraphicsProtocol::Halfblocks
    }
}
