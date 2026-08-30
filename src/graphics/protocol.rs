use crate::config::ImageProtocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsProtocol {
    Kitty,
    Halfblocks,
    None,
}

impl GraphicsProtocol {
    pub fn is_kitty(&self) -> bool {
        matches!(self, Self::Kitty)
    }

    pub fn is_halfblocks(&self) -> bool {
        matches!(self, Self::Halfblocks)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Kitty => "kitty",
            Self::Halfblocks => "halfblocks",
            Self::None => "none",
        }
    }
}

pub fn resolve_protocol(configured: ImageProtocol) -> GraphicsProtocol {
    match configured {
        ImageProtocol::Off => GraphicsProtocol::None,
        ImageProtocol::Kitty => GraphicsProtocol::Kitty,
        ImageProtocol::Halfblocks => GraphicsProtocol::Halfblocks,
        ImageProtocol::Auto => super::detect::detect_terminal_protocol(),
    }
}
