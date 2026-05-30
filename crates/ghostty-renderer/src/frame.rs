//! CPU-side draw frame orchestration before GPU backends land.

use crate::cell::CellText;
use crate::cell::CellBgDraw;
use crate::cells::{rebuild_cells, CellSnapshot};
use crate::damage::{DamageRect, DamageState};
use crate::size::GridSize;

/// Result of preparing a draw frame from terminal snapshot + damage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramePrep {
    pub grid: GridSize,
    pub dirty_rects: Vec<DamageRect>,
    pub populated_cells: usize,
    pub text_cells: Vec<CellText>,
    pub bg_cells: Vec<CellBgDraw>,
}

/// Merge snapshot into damage and describe the next draw pass.
pub fn prepare_draw_frame(snapshot: &CellSnapshot, damage: &mut DamageState) -> FramePrep {
    rebuild_cells(snapshot, damage);
    let dirty = damage.take();
    let populated_cells = snapshot.codepoints.iter().filter(|cp| cp.is_some()).count();
    FramePrep {
        grid: snapshot.grid,
        dirty_rects: if dirty.full_redraw {
            vec![DamageRect::full_screen(snapshot.grid)]
        } else {
            dirty.rects
        },
        populated_cells,
        text_cells: Vec::new(),
        bg_cells: Vec::new(),
    }
}

/// Mark damage clean after a frame was presented (CPU stub until GPU exists).
pub fn finish_draw_frame(damage: &mut DamageState) {
    *damage = DamageState::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_counts_populated_cells() {
        let grid = GridSize {
            columns: 4,
            rows: 2,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, b'A' as u32);
        snap.set(1, 0, b'B' as u32);
        let mut damage = DamageState::default();
        let prep = prepare_draw_frame(&snap, &mut damage);
        assert_eq!(prep.populated_cells, 2);
        assert!(!prep.dirty_rects.is_empty());
    }

    #[test]
    fn finish_clears_damage() {
        let mut damage = DamageState::default();
        damage.mark_full();
        finish_draw_frame(&mut damage);
        assert!(!damage.is_dirty());
    }
}
