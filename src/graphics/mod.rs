pub mod cache;
pub mod detect;
pub mod halfblocks;
pub mod kitty;
pub mod protocol;

pub use cache::{get_cached_image_path, image_cache_path, save_cached_image};
pub use detect::detect_terminal_protocol;
pub use halfblocks::{render_halfblock_lines, RgbPixel};
pub use protocol::{resolve_protocol, GraphicsProtocol};
