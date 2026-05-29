//! Damage / redraw tracking (`src/renderer/` damage state subset).

use crate::size::{CellCount, GridSize};

/// Dirty region in grid coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageRect {
    pub x: CellCount,
    pub y: CellCount,
    pub width: CellCount,
    pub height: CellCount,
}

impl DamageRect {
    pub fn full_screen(size: GridSize) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.columns,
            height: size.rows,
        }
    }

    pub fn contains(&self, x: CellCount, y: CellCount) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width)
            && y < self.y.saturating_add(self.height)
    }
}

/// Accumulated redraw state for a surface renderer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DamageState {
    pub full_redraw: bool,
    pub rects: Vec<DamageRect>,
}

impl DamageState {
    pub fn mark_full(&mut self) {
        self.full_redraw = true;
        self.rects.clear();
    }

    pub fn mark_rect(&mut self, rect: DamageRect) {
        if self.full_redraw {
            return;
        }
        self.rects.push(rect);
    }

    pub fn take(&mut self) -> DamageState {
        core::mem::take(self)
    }

    pub fn is_dirty(&self) -> bool {
        self.full_redraw || !self.rects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_redraw_clears_partial() {
        let mut damage = DamageState::default();
        damage.mark_rect(DamageRect {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        });
        damage.mark_full();
        assert!(damage.full_redraw);
        assert!(damage.rects.is_empty());
    }

    #[test]
    fn rect_contains_point() {
        let rect = DamageRect {
            x: 2,
            y: 3,
            width: 4,
            height: 5,
        };
        assert!(rect.contains(3, 4));
        assert!(!rect.contains(10, 10));
    }
}
