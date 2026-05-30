//! Loaded glyph metadata (`src/font/Glyph.zig`).

/// A single glyph rasterized into the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Glyph {
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub atlas_x: u32,
    pub atlas_y: u32,
}

impl Glyph {
    pub const fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            offset_x: 0,
            offset_y: 0,
            atlas_x: 0,
            atlas_y: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRenderError {
    InvalidCodepoint,
    UnsupportedCodepoint,
    Atlas(crate::AtlasError),
    Platform,
}

impl From<crate::AtlasError> for GlyphRenderError {
    fn from(err: crate::AtlasError) -> Self {
        Self::Atlas(err)
    }
}
