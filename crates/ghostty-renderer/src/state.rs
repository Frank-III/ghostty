//! Renderer input state shared with the surface.
//!
//! Port target: `src/renderer/State.zig` (preedit/mouse; terminal pointer deferred).

use crate::size::CellCount;

/// Dead-key / IME preedit overlay (`State.Preedit`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Preedit {
    pub codepoints: Vec<PreeditCodepoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreeditCodepoint {
    pub codepoint: u32,
    pub wide: bool,
}

impl Preedit {
    pub fn width(&self) -> usize {
        self.codepoints
            .iter()
            .map(|cp| if cp.wide { 2 } else { 1 })
            .sum()
    }

    /// Start/end grid columns and codepoint offset when clipped to `max` (inclusive).
    pub fn range(
        &self,
        start: CellCount,
        max: CellCount,
    ) -> PreeditRange {
        let max_width = max.saturating_sub(start) + 1;
        let mut w: CellCount = 0;
        let mut cp_offset = 0usize;
        for i in 0..self.codepoints.len() {
            let reverse_i = self.codepoints.len() - i - 1;
            let cp = self.codepoints[reverse_i];
            w += if cp.wide { 2 } else { 1 };
            if w > max_width {
                cp_offset = reverse_i;
                break;
            }
        }
        let end = if w > 0 {
            start.saturating_add(w - 1)
        } else {
            start
        };
        let start_offset = if end > max { end - max } else { 0 };
        PreeditRange {
            start: start.saturating_sub(start_offset),
            end: end.saturating_sub(start_offset),
            cp_offset,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreeditRange {
    pub start: CellCount,
    pub end: CellCount,
    pub cp_offset: usize,
}

/// Mouse fields needed by the renderer (`State.Mouse`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RendererMouse {
    pub point: Option<GridPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPoint {
    pub x: CellCount,
    pub y: CellCount,
}

/// Inputs guarded by the renderer mutex in Zig (`renderer.State`).
///
/// The struct itself is not thread-safe; only the referenced terminal data is
/// protected by the mutex in the Zig implementation.
#[derive(Clone, Debug, Default)]
pub struct RendererState {
    pub preedit: Option<Preedit>,
    pub mouse: RendererMouse,
}

#[cfg(test)]
mod tests {
    use super::*;

    const HANGUL_GA: u32 = 0xAC00;

    #[test]
    fn preedit_range_single_ascii() {
        let p = Preedit {
            codepoints: vec![PreeditCodepoint {
                codepoint: b'a' as u32,
                wide: false,
            }],
        };
        let range = p.range(2, 9);
        assert_eq!(range.start, 2);
        assert_eq!(range.end, 2);
        assert_eq!(range.cp_offset, 0);
    }

    #[test]
    fn preedit_range_wide_hangul() {
        let p = Preedit {
            codepoints: vec![PreeditCodepoint {
                codepoint: HANGUL_GA,
                wide: true,
            }],
        };
        let range = p.range(2, 9);
        assert_eq!(range.start, 2);
        assert_eq!(range.end, 3);
        assert_eq!(range.cp_offset, 0);
    }

    #[test]
    fn preedit_range_shifts_at_right_edge() {
        let p = Preedit {
            codepoints: vec![PreeditCodepoint {
                codepoint: HANGUL_GA,
                wide: true,
            }],
        };
        let range = p.range(9, 9);
        assert_eq!(range.start, 8);
        assert_eq!(range.end, 9);
        assert_eq!(range.cp_offset, 0);
    }
}
