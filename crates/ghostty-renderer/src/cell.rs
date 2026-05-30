//! Cell draw types and glyph layout heuristics.
//!
//! Port target: `src/renderer/cell.zig`, `src/renderer/metal/shaders.zig` (`CellText` / `CellBg`).

use ghostty_foundation::unicode::is_symbol;

use crate::color::ShaderRgba;

/// Vertex buffer key for foreground layers (`cell.Key`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKey {
    Bg,
    Text,
    Underline,
    Strikethrough,
    Overline,
}

/// Background cell shader parameter (`shaderpkg.CellBg`).
pub type CellBg = ShaderRgba;

/// Glyph atlas selector (`CellText.Atlas`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CellAtlas {
    #[default]
    Grayscale = 0,
    Color = 1,
}

/// Packed flags on `CellText` (`CellText.bools`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellTextBools {
    bits: u8,
}

impl CellTextBools {
    pub fn new(no_min_contrast: bool, is_cursor_glyph: bool) -> Self {
        let mut bits = 0u8;
        if no_min_contrast {
            bits |= 1;
        }
        if is_cursor_glyph {
            bits |= 2;
        }
        Self { bits }
    }

    pub fn no_min_contrast(self) -> bool {
        self.bits & 1 != 0
    }

    pub fn is_cursor_glyph(self) -> bool {
        self.bits & 2 != 0
    }
}

/// Background cell instance for GPU quads (`shaderpkg.CellBg` + grid position).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellBgDraw {
    pub grid_pos: [u16; 2],
    pub color: CellBg,
}

/// Foreground cell shader parameter (`shaderpkg.CellText`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellText {
    pub glyph_pos: [u32; 2],
    pub glyph_size: [u32; 2],
    pub bearings: [i16; 2],
    pub grid_pos: [u16; 2],
    pub color: ShaderRgba,
    pub atlas: CellAtlas,
    pub bools: CellTextBools,
}

/// Minimal cell view for `constraint_width` tests without the terminal crate.
#[derive(Clone, Copy, Debug)]
pub struct CellView {
    pub codepoint: u32,
    pub grid_width: u8,
}

/// Returns true for U+2588 FULL BLOCK (`cell.isCovering`).
pub fn is_covering(cp: u32) -> bool {
    cp == 0x2588
}

/// Whether min contrast should be disabled (blocks, Powerline, box drawing, etc.).
pub fn no_min_contrast(cp: u32) -> bool {
    is_graphics_element(cp)
}

/// Constraint width for glyph shaping (`cell.constraintWidth`).
pub fn constraint_width(cells: &[CellView], x: usize, cols: usize) -> u8 {
    let cell = cells[x];
    let cp = cell.codepoint;
    let grid_width = cell.grid_width;
    if grid_width > 1 {
        return grid_width;
    }
    if !is_symbol(cp) {
        return grid_width;
    }
    if x == cols - 1 {
        return 1;
    }
    if x > 0 {
        let prev_cp = cells[x - 1].codepoint;
        if is_symbol(prev_cp) && !is_graphics_element(prev_cp) {
            return 1;
        }
    }
    let next_cp = cells[x + 1].codepoint;
    if next_cp == 0 || is_space(next_cp) {
        return 2;
    }
    1
}

fn is_space(ch: u32) -> bool {
    matches!(ch, 0x0020 | 0x2002)
}

fn is_graphics_element(ch: u32) -> bool {
    is_box_drawing(ch) || is_block_element(ch) || is_legacy_computing(ch) || is_powerline(ch)
}

fn is_box_drawing(ch: u32) -> bool {
    (0x2500..=0x257F).contains(&ch)
}

fn is_block_element(ch: u32) -> bool {
    (0x2580..=0x259F).contains(&ch)
}

fn is_legacy_computing(ch: u32) -> bool {
    (0x1FB00..=0x1FBFF).contains(&ch) || (0x1CC00..=0x1CEBF).contains(&ch)
}

fn is_powerline(ch: u32) -> bool {
    (0xE0B0..=0xE0D7).contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn cell_text_size_matches_zig() {
        assert_eq!(size_of::<CellText>(), 32);
    }

    #[test]
    fn covering_block() {
        assert!(is_covering(0x2588));
        assert!(!is_covering(b'a' as u32));
    }

    #[test]
    fn constraint_symbol_before_space() {
        let cells = [
            CellView {
                codepoint: 0xE8AF,
                grid_width: 1,
            },
            CellView {
                codepoint: b' ' as u32,
                grid_width: 1,
            },
            CellView {
                codepoint: 0,
                grid_width: 1,
            },
            CellView {
                codepoint: 0,
                grid_width: 1,
            },
        ];
        assert_eq!(constraint_width(&cells, 0, 4), 2);
    }

    #[test]
    fn constraint_symbol_at_row_end() {
        let cells = [
            CellView {
                codepoint: 0,
                grid_width: 1,
            },
            CellView {
                codepoint: 0,
                grid_width: 1,
            },
            CellView {
                codepoint: 0,
                grid_width: 1,
            },
            CellView {
                codepoint: 0xE8AF,
                grid_width: 1,
            },
        ];
        assert_eq!(constraint_width(&cells, 3, 4), 1);
    }
}
