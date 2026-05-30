//! CPU-staged draw passes until Metal/OpenGL/WebGL pipelines land.
//!
//! Backends record instance counts and grid bounds; real GPU work replaces the
//! default [`GraphicsApi`] implementations in `metal` / `opengl` / `webgl`.

use crate::cell::{CellBgDraw, CellText};
use crate::frame::FramePrep;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;
use crate::uniforms::FrameUniforms;

/// Counters from the most recent background + text draw passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrawPassStats {
    pub bg_instances: usize,
    pub text_instances: usize,
    pub cursor_instances: usize,
}

/// Issue background then text draws for a prepared frame.
pub fn issue_draw_pass<A: GraphicsApi>(
    api: &mut A,
    size: &Size,
    prep: &FramePrep,
    uniforms: &FrameUniforms,
) -> Result<DrawPassStats, GraphicsError> {
    api.draw_background_pass(size, &prep.bg_cells, uniforms)?;
    let mut text_cells = prep.text_cells.clone();
    if let Some(cursor) = &prep.cursor_cell {
        match prep.cursor_uniforms {
            Some(_) => text_cells.insert(0, *cursor),
            None => text_cells.push(*cursor),
        }
    }
    api.draw_text_pass(size, &text_cells, prep.cursor_uniforms.as_ref())?;
    Ok(DrawPassStats {
        bg_instances: prep.bg_cells.len(),
        text_instances: prep.text_cells.len(),
        cursor_instances: usize::from(prep.cursor_cell.is_some()),
    })
}

/// Default background pass: validate instances against grid (stub records counts in API).
pub fn draw_background_pass_stub(size: &Size, cells: &[CellBgDraw]) -> Result<(), GraphicsError> {
    let grid = size.grid();
    for cell in cells {
        if cell.grid_pos[0] >= grid.columns || cell.grid_pos[1] >= grid.rows {
            return Err(GraphicsError::InvalidDrawInstance);
        }
    }
    Ok(())
}

/// Default text pass: validate grid positions (atlas sampling deferred to GPU).
pub fn draw_text_pass_stub(size: &Size, cells: &[CellText]) -> Result<(), GraphicsError> {
    let grid = size.grid();
    for cell in cells {
        if cell.grid_pos[0] >= grid.columns || cell.grid_pos[1] >= grid.rows {
            return Err(GraphicsError::InvalidDrawInstance);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellBgDraw;
    use crate::color::shader_rgba;
    use crate::color::Rgb;
    use crate::frame::prepare_draw_frame;
    use crate::generic::StubGraphicsApi;
    use crate::size::{CellSize, GridSize, Padding, ScreenSize, Size};

    fn test_size() -> Size {
        Size {
            screen: ScreenSize {
                width: 64,
                height: 32,
            },
            cell: CellSize {
                width: 8,
                height: 16,
            },
            padding: Padding::default(),
        }
    }

    #[test]
    fn issue_draw_pass_validates_grid() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let mut snap = crate::cells::CellSnapshot::empty(grid);
        snap.set(0, 0, b'A' as u32);
        let mut damage = crate::damage::DamageState::default();
        let mut prep = prepare_draw_frame(&snap, &mut damage);
        prep.bg_cells.push(CellBgDraw {
            grid_pos: [0, 0],
            color: shader_rgba(Rgb::new(0, 0, 0), 0xff),
        });
        let mut api = StubGraphicsApi;
        let uniforms = crate::uniforms::FrameUniforms::from_size(
            &test_size(),
            crate::color::shader_rgba(crate::color::Rgb::new(0, 0, 0), 0xff),
        );
        issue_draw_pass(&mut api, &test_size(), &prep, &uniforms).unwrap();
    }
}
