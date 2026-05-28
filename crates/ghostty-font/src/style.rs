//! Font style and emoji presentation. Port target: `src/font/main.zig`.

/// Style bitmask for a font family face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Style {
    #[default]
    Regular = 0,
    Bold = 1,
    Italic = 2,
    BoldItalic = 3,
}

/// Emoji presentation (text vs emoji).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Presentation {
    Text = 0,
    Emoji = 1,
}
