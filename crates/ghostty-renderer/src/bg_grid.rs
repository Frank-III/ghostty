//! Pack sparse background instances into a dense grid buffer for GPU shaders.

use crate::cell::CellBg;
use crate::cell::CellBgDraw;
use crate::color::{shader_rgba, Rgb};
use crate::size::GridSize;

/// Dense per-cell background colors (`shaderpkg.CellBg` per grid slot).
pub fn pack_bg_grid(grid: GridSize, default_bg: Rgb, sparse: &[CellBgDraw]) -> Vec<CellBg> {
    let len = usize::from(grid.columns) * usize::from(grid.rows);
    let default = shader_rgba(default_bg, 0xff);
    let mut out = vec![default; len];
    for cell in sparse {
        let x = usize::from(cell.grid_pos[0]);
        let y = usize::from(cell.grid_pos[1]);
        if x >= usize::from(grid.columns) || y >= usize::from(grid.rows) {
            continue;
        }
        let idx = y * usize::from(grid.columns) + x;
        if let Some(slot) = out.get_mut(idx) {
            *slot = cell.color;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;

    #[test]
    fn pack_fills_default_and_overrides() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let default = Rgb::new(0, 0, 0);
        let sparse = vec![CellBgDraw {
            grid_pos: [1, 0],
            color: shader_rgba(Rgb::new(0xff, 0, 0), 0xff),
        }];
        let packed = pack_bg_grid(grid, default, &sparse);
        assert_eq!(packed.len(), 2);
        assert_eq!(packed[0], shader_rgba(default, 0xff));
        assert_eq!(packed[1][0..3], [0xff, 0, 0]);
    }
}
