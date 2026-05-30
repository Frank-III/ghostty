//! Text shaping hooks (`src/font/shaper/` port target).

use ghostty_foundation::unicode::codepoint_width;

use crate::discovery::DiscoveredFont;
use crate::shaper::{HarfBuzzShaper, ShapedGlyph};

/// Map VT wide-cell tag (`page_types::Wide` as u8) to grid columns for drawing.
pub fn grid_columns_from_wide_raw(raw: u8) -> u8 {
    match raw {
        1 => 2,
        2 => 0,
        _ => 1,
    }
}

#[inline]
pub fn grapheme_display_width(cp: u32) -> u8 {
    codepoint_width(cp).clamp(0, 2) as u8
}

pub fn cluster_display_width(codepoints: &[u32]) -> u8 {
    codepoints
        .iter()
        .map(|&cp| grapheme_display_width(cp))
        .fold(0u8, u8::saturating_add)
}

/// Shaping session: HarfBuzz (rustybuzz) when a font path is available.
#[derive(Debug)]
pub struct ShapingSession {
    harfbuzz: Option<HarfBuzzShaper>,
    font_size: f32,
}

impl ShapingSession {
    pub fn new() -> Self {
        Self {
            harfbuzz: None,
            font_size: 12.0,
        }
    }

    pub fn open_from_discovered(discovered: &DiscoveredFont, font_size: f32) -> Self {
        let harfbuzz = discovered.path.as_deref().and_then(HarfBuzzShaper::open);
        Self {
            harfbuzz,
            font_size,
        }
    }

    pub fn uses_harfbuzz(&self) -> bool {
        self.harfbuzz.as_ref().is_some_and(|h| h.enabled())
    }

    /// Shape a codepoint run; returns empty when HarfBuzz is unavailable.
    pub fn shape(&self, codepoints: &[u32]) -> Vec<ShapedGlyph> {
        self.harfbuzz
            .as_ref()
            .map(|h| h.shape(codepoints, self.font_size))
            .unwrap_or_default()
    }
}

impl Default for ShapingSession {
    fn default() -> Self {
        Self::new()
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
    }

    #[test]
    fn session_without_font_has_no_shaper() {
        let session = ShapingSession::new();
        assert!(!session.uses_harfbuzz());
        assert!(session.shape(&[b'a' as u32]).is_empty());
    }
}
