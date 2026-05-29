//! Terminal cell snapshot / rebuild skeleton (`src/renderer/` cell grid path).

use crate::damage::{DamageRect, DamageState};
use crate::size::GridSize;

/// Read-only cell snapshot for renderer rebuild (content deferred to Zig FFI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellSnapshot {
    pub grid: GridSize,
    pub codepoints: Vec<Option<u32>>,
}

impl CellSnapshot {
    pub fn empty(grid: GridSize) -> Self {
        let len = usize::from(grid.columns) * usize::from(grid.rows);
        Self {
            grid,
            codepoints: vec![None; len],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, cp: u32) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.codepoints.get_mut(idx) {
            *slot = Some(cp);
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
