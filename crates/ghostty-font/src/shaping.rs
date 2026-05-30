//! Text shaping hooks (`src/font/shaper/` port target).
//!
//! Full HarfBuzz integration remains deferred; terminal grid width uses the
//! same Unicode properties tables as Zig via `ghostty_foundation`.

use ghostty_foundation::unicode::codepoint_width;

/// Map VT wide-cell tag (`page_types::Wide` as u8) to grid columns for drawing.
/// Returns `0` for spacer tails (no foreground glyph).
pub fn grid_columns_from_wide_raw(raw: u8) -> u8 {
    match raw {
        1 => 2,
        2 => 0,
        _ => 1,
    }
}

/// Display width of a single codepoint in terminal cells (0, 1, or 2).
#[inline]
pub fn grapheme_display_width(cp: u32) -> u8 {
    codepoint_width(cp).clamp(0, 2) as u8
}

/// Cluster width for a UTF-32 codepoint slice (sum of per-codepoint widths).
pub fn cluster_display_width(codepoints: &[u32]) -> u8 {
    codepoints
        .iter()
        .map(|&cp| grapheme_display_width(cp))
        .fold(0u8, u8::saturating_add)
}

/// Placeholder for future HarfBuzz-backed shaping sessions.
#[derive(Debug, Default)]
pub struct ShapingSession {
    harfbuzz_enabled: bool,
}

impl ShapingSession {
    pub fn new() -> Self {
        Self {
            harfbuzz_enabled: false,
        }
    }

    /// When false, [`grapheme_display_width`] is used for layout hints.
    pub fn uses_harfbuzz(&self) -> bool {
        self.harfbuzz_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_columns_from_wide() {
        assert_eq!(grid_columns_from_wide_raw(0), 1);
        assert_eq!(grid_columns_from_wide_raw(1), 2);
        assert_eq!(grid_columns_from_wide_raw(2), 0);
        assert_eq!(grid_columns_from_wide_raw(3), 1);
    }

    #[test]
    fn ascii_width_one() {
        assert_eq!(grapheme_display_width(b'a' as u32), 1);
    }

    #[test]
    fn wide_char_width_two() {
        assert_eq!(grapheme_display_width(0x4e00), 2);
    }

    #[test]
    fn cluster_width_sums() {
        assert_eq!(cluster_display_width(&[b'a' as u32, 0x4e00]), 3);
    }
}
