//! Codepoint width helpers built on the generated properties LUT.
//!
//! Mirrors `src/simd/codepoint_width.zig` (non-SIMD path) and
//! `src/terminal/Terminal.zig` width lookups via `unicode.table`.

use super::props_table;

/// Display width for a codepoint in terminal cells.
///
/// ASCII codepoints (`<= 0xFF`) are always width 1. Out-of-range codepoints
/// use [`props_table::Properties::OUT_OF_RANGE`] (width 1).
pub fn codepoint_width(cp: u32) -> u8 {
    if cp <= 0xFF {
        return 1;
    }
    props_table::get(cp).width
}

/// Whether the codepoint does not contribute to grapheme-cluster width.
pub fn width_zero_in_grapheme(cp: u32) -> bool {
    props_table::get(cp).width_zero_in_grapheme
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_width_basic() {
        assert_eq!(codepoint_width(b'a' as u32), 1);
        assert_eq!(codepoint_width(0x100), 1); // Ā
        assert_eq!(codepoint_width(0x3400), 2); // 㐀
        assert_eq!(codepoint_width(0x2E3A), 2); // ⸺
        assert_eq!(codepoint_width(0x1F1E6), 2); // 🇦
        assert_eq!(codepoint_width(0x4E00), 2); // 一
        assert_eq!(codepoint_width(0xF900), 2); // 豈
        assert_eq!(codepoint_width(0x20000), 2); // 𠀀
        assert_eq!(codepoint_width(0x30000), 2); // 𠀀
    }

    #[test]
    fn out_of_range_width() {
        assert_eq!(codepoint_width(0x110000), 1);
    }

    #[test]
    fn combining_mark_zero_in_grapheme() {
        assert!(width_zero_in_grapheme(0x0300)); // combining grave accent
    }
}
