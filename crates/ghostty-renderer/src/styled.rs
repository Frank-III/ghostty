//! Styled terminal cells for renderer rebuild (`cell.Contents` subset).

use crate::color::Rgb;
use crate::size::GridSize;

/// Minimal cell style for renderer rebuild (full Style deferred to VT FFI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellStyle {
    pub foreground: Option<Rgb>,
    pub background: Option<Rgb>,
    pub bold: bool,
    pub italic: bool,
    pub inverse: bool,
}

/// One visible terminal cell with optional styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyledCell {
    pub codepoint: u32,
    pub style: CellStyle,
}

/// Grid of styled cells for GPU buffer generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledCellSnapshot {
    pub grid: GridSize,
    pub cells: Vec<Option<StyledCell>>,
}

impl StyledCellSnapshot {
    pub fn empty(grid: GridSize) -> Self {
        let len = usize::from(grid.columns) * usize::from(grid.rows);
        Self {
            grid,
            cells: vec![None; len],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, cell: StyledCell) {
        let idx = Self::index(self.grid, x, y);
        if let Some(slot) = self.cells.get_mut(idx) {
            *slot = Some(cell);
        }
    }

    pub fn populated_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_some()).count()
    }

    fn index(grid: GridSize, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(grid.columns) + usize::from(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_cell_set_and_count() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let mut snap = StyledCellSnapshot::empty(grid);
        snap.set(
            0,
            0,
            StyledCell {
                codepoint: b'A' as u32,
                style: CellStyle {
                    bold: true,
                    ..Default::default()
                },
            },
        );
        assert_eq!(snap.populated_count(), 1);
    }
}
