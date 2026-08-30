use super::protocol::GraphicsProtocol;
use std::env;

pub fn detect_terminal_protocol() -> GraphicsProtocol {
    if env::var("KITTY_WINDOW_ID").is_ok()
        || env::var("GHOSTTY_RESOURCES_DIR").is_ok()
        || env::var("TERM").map(|t| t == "xterm-kitty" || t.contains("kitty") || t.contains("ghostty")).unwrap_or(false)
    {
        GraphicsProtocol::Kitty
    } else {
        GraphicsProtocol::Halfblocks
    }
}
