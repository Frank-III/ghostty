//! Layout sizes and coordinate conversion.
//!
//! Port target: `src/renderer/size.zig`.

/// Cell grid dimensions in rows/columns (`renderer.GridSize`).
pub type CellCount = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GridSize {
    pub columns: CellCount,
    pub rows: CellCount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CellSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Padding {
    pub top: u32,
    pub bottom: u32,
    pub right: u32,
    pub left: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaddingBalance {
    /// Padding is applied as specified; `balance_padding` must not use this mode.
    False,
    /// Cap top padding; shift excess to the bottom.
    True,
    /// Center the grid with equal padding on all sides.
    Equal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub screen: ScreenSize,
    pub cell: CellSize,
    pub padding: Padding,
}

impl ScreenSize {
    pub fn sub_padding(self, padding: Padding) -> ScreenSize {
        ScreenSize {
            width: self.width.saturating_sub(padding.left + padding.right),
            height: self.height.saturating_sub(padding.top + padding.bottom),
        }
    }
}

impl GridSize {
    pub fn init(screen: ScreenSize, cell: CellSize) -> GridSize {
        let mut g = GridSize::default();
        g.update(screen, cell);
        g
    }

    pub fn update(&mut self, screen: ScreenSize, cell: CellSize) {
        let cell_width = cell.width as f32;
        let cell_height = cell.height as f32;
        let screen_width = screen.width as f32;
        let screen_height = screen.height as f32;
        let calc_cols = (screen_width / cell_width) as CellCount;
        let calc_rows = (screen_height / cell_height) as CellCount;
        self.columns = calc_cols.max(1);
        self.rows = calc_rows.max(1);
    }

    pub fn equals(self, other: GridSize) -> bool {
        self.columns == other.columns && self.rows == other.rows
    }
}

impl Padding {
    pub fn balanced(screen: ScreenSize, grid: GridSize, cell: CellSize) -> Padding {
        let cell_width = cell.width as f32;
        let cell_height = cell.height as f32;
        let grid_width = f32::from(grid.columns) * cell_width;
        let grid_height = f32::from(grid.rows) * cell_height;
        let space_right = screen.width as f32 - grid_width;
        let space_bot = screen.height as f32 - grid_height;
        let padding_right = (space_right / 2.0).floor();
        let padding_left = padding_right;
        let padding_bot = (space_bot / 2.0).floor();
        let padding_top = padding_bot;
        Padding {
            top: padding_top.max(0.0) as u32,
            bottom: padding_bot.max(0.0) as u32,
            right: padding_right.max(0.0) as u32,
            left: padding_left.max(0.0) as u32,
        }
    }

    pub fn eql(self, other: Padding) -> bool {
        self == other
    }
}

impl Size {
    pub fn grid(self) -> GridSize {
        GridSize::init(self.screen.sub_padding(self.padding), self.cell)
    }

    pub fn terminal(self) -> ScreenSize {
        self.screen.sub_padding(self.padding)
    }

    pub fn balance_padding(&mut self, explicit: Padding, mode: PaddingBalance) {
        self.padding = explicit;
        self.padding = Padding::balanced(self.screen, self.grid(), self.cell);

        if let PaddingBalance::True = mode {
            let max_top = (explicit.left + explicit.right + self.cell.width) / 2;
            let vshift = self.padding.top.saturating_sub(max_top);
            self.padding.top -= vshift;
            self.padding.bottom += vshift;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coordinate {
    Surface { x: f64, y: f64 },
    Terminal { x: f64, y: f64 },
    Grid { x: CellCount, y: CellCount },
}

impl Coordinate {
    pub fn convert(self, to: CoordinateSpace, size: Size) -> Coordinate {
        if self.space() == to {
            return self;
        }
        let surface = self.to_surface(size);
        match to {
            CoordinateSpace::Surface => Coordinate::Surface {
                x: surface.x,
                y: surface.y,
            },
            CoordinateSpace::Terminal => Coordinate::Terminal {
                x: surface.x - f64::from(size.padding.left),
                y: surface.y - f64::from(size.padding.top),
            },
            CoordinateSpace::Grid => {
                let term = Coordinate::Surface {
                    x: surface.x,
                    y: surface.y,
                }
                .convert(CoordinateSpace::Terminal, size);
                let Coordinate::Terminal { x, y } = term else {
                    unreachable!()
                };
                let grid = size.grid();
                let cell_width = size.cell.width as f64;
                let cell_height = size.cell.height as f64;
                let clamped_x = x.max(0.0);
                let clamped_y = y.max(0.0);
                let col = (clamped_x / cell_width) as CellCount;
                let row = (clamped_y / cell_height) as CellCount;
                let clamped_col = col.min(grid.columns.saturating_sub(1));
                let clamped_row = row.min(grid.rows.saturating_sub(1));
                Coordinate::Grid {
                    x: clamped_col,
                    y: clamped_row,
                }
            }
        }
    }

    fn space(self) -> CoordinateSpace {
        match self {
            Coordinate::Surface { .. } => CoordinateSpace::Surface,
            Coordinate::Terminal { .. } => CoordinateSpace::Terminal,
            Coordinate::Grid { .. } => CoordinateSpace::Grid,
        }
    }

    fn to_surface(self, size: Size) -> SurfacePoint {
        match self {
            Coordinate::Surface { x, y } => SurfacePoint { x, y },
            Coordinate::Terminal { x, y } => SurfacePoint {
                x: x + f64::from(size.padding.left),
                y: y + f64::from(size.padding.top),
            },
            Coordinate::Grid { x, y } => {
                let col = f64::from(x);
                let row = f64::from(y);
                SurfacePoint {
                    x: col * f64::from(size.cell.width) + f64::from(size.padding.left),
                    y: row * f64::from(size.cell.height) + f64::from(size.padding.top),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateSpace {
    Surface,
    Terminal,
    Grid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SurfacePoint {
    x: f64,
    y: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_padding_equal_is_symmetric() {
        let mut size = Size {
            screen: ScreenSize {
                width: 1050,
                height: 850,
            },
            cell: CellSize {
                width: 10,
                height: 20,
            },
            padding: Padding::default(),
        };
        size.balance_padding(
            Padding {
                top: 4,
                bottom: 4,
                left: 4,
                right: 4,
            },
            PaddingBalance::Equal,
        );
        assert_eq!(size.padding.left, size.padding.right);
        assert_eq!(size.padding.top, size.padding.bottom);
        assert!(size.padding.top > 0);
    }

    #[test]
    fn balance_padding_true_shifts_excess_top() {
        let mut size = Size {
            screen: ScreenSize {
                width: 1090,
                height: 1070,
            },
            cell: CellSize {
                width: 20,
                height: 40,
            },
            padding: Padding::default(),
        };
        size.balance_padding(Padding::default(), PaddingBalance::True);
        assert_eq!(size.padding.left, size.padding.right);
        assert!(size.padding.top < size.padding.bottom);
        assert_eq!(size.padding.top, 10);
        assert_eq!(size.padding.bottom, 20);
    }

    #[test]
    fn grid_size_update_exact() {
        let mut grid = GridSize::default();
        grid.update(
            ScreenSize {
                width: 100,
                height: 40,
            },
            CellSize {
                width: 5,
                height: 10,
            },
        );
        assert_eq!(grid.columns, 20);
        assert_eq!(grid.rows, 4);
    }

    #[test]
    fn coordinate_surface_to_grid() {
        let size = Size {
            screen: ScreenSize {
                width: 100,
                height: 100,
            },
            cell: CellSize {
                width: 5,
                height: 10,
            },
            padding: Padding::default(),
        };
        let actual = Coordinate::Surface { x: 6.0, y: 10.0 }.convert(CoordinateSpace::Grid, size);
        assert_eq!(actual, Coordinate::Grid { x: 1, y: 1 });
    }
}
