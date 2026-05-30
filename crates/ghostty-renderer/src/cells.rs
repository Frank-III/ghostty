//! Terminal cell snapshot / rebuild skeleton (`src/renderer/` cell grid path).

use crate::damage::{DamageRect, DamageState};
use crate::size::GridSize;

/// Read-only cell snapshot for renderer rebuild (content deferred to Zig FFI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellSnapshot {
    pub grid: GridSize,
    pub codepoints: Vec<Option<u32>>,
    pub foregrounds: Vec<Option<crate::color::Rgb>>,
    pub backgrounds: Vec<Option<crate::color::Rgb>>,
    /// VT wide tag per cell (`page_types::Wide` as u8); `0` = narrow.
    pub wide_raw: Vec<u8>,
}

impl CellSnapshot {
    pub fn empty(grid: GridSize) -> Self {
        let len = usize::from(grid.columns) * usize::from(grid.rows);
        Self {
            grid,
            codepoints: vec![None; len],
            foregrounds: vec![None; len],
            backgrounds: vec![None; len],
            wide_raw: vec![0; len],
        }
    }

    pub fn set_wide(&mut self, x: u16, y: u16, raw: u8) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.wide_raw.get_mut(idx) {
            *slot = raw;
        }
    }

    pub fn grid_columns_at(&self, idx: usize) -> u8 {
        ghostty_font::grid_columns_from_wide_raw(*self.wide_raw.get(idx).unwrap_or(&0))
    }

    pub fn skip_text_at(&self, idx: usize) -> bool {
        self.wide_raw.get(idx).copied().unwrap_or(0) == 2
    }

    pub fn set(&mut self, x: u16, y: u16, cp: u32) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.codepoints.get_mut(idx) {
            *slot = Some(cp);
        }
    }

    pub fn set_foreground(&mut self, x: u16, y: u16, color: crate::color::Rgb) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.foregrounds.get_mut(idx) {
            *slot = Some(color);
        }
    }

    pub fn set_background(&mut self, x: u16, y: u16, color: crate::color::Rgb) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.backgrounds.get_mut(idx) {
            *slot = Some(color);
        }
    }

    fn index(grid: GridSize, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(grid.columns) + usize::from(x)
    }
}

/// Rebuild pass: apply snapshot into damage for the next draw.
pub fn rebuild_cells(snapshot: &CellSnapshot, damage: &mut DamageState) {
    damage.mark_rect(DamageRect::full_screen(snapshot.grid));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_marks_full_screen() {
        let grid = GridSize {
            columns: 80,
            rows: 24,
        };
        let snap = CellSnapshot::empty(grid);
        let mut damage = DamageState::default();
        rebuild_cells(&snap, &mut damage);
        assert!(damage.is_dirty());
    }
}
