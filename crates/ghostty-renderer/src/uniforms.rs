//! GPU uniform block matching `src/renderer/metal/shaders.zig` (subset).

use crate::color::ShaderRgba;
use crate::size::Size;

/// Cursor uniforms for block-cursor text tint in the text shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorUniforms {
    pub grid_pos: [u16; 2],
    pub color: ShaderRgba,
    pub wide: bool,
}

/// Frame uniforms shared by bg/text pipelines (CPU mirror; GPU upload deferred).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameUniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub grid_columns: f32,
    pub grid_rows: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub bg_color: ShaderRgba,
}

impl FrameUniforms {
    pub fn from_size(size: &Size, bg_color: ShaderRgba) -> Self {
        let grid = size.grid();
        Self {
            screen_width: size.screen.width as f32,
            screen_height: size.screen.height as f32,
            cell_width: size.cell.width as f32,
            cell_height: size.cell.height as f32,
            grid_columns: grid.columns as f32,
            grid_rows: grid.rows as f32,
            padding_left: size.padding.left as f32,
            padding_top: size.padding.top as f32,
            bg_color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::shader_rgba;
    use crate::color::Rgb;
    use crate::size::{CellSize, GridSize, Padding, ScreenSize, Size};

    #[test]
    fn frame_uniforms_from_size() {
        let size = Size {
            screen: ScreenSize {
                width: 640,
                height: 480,
            },
            cell: CellSize {
                width: 8,
                height: 16,
            },
            padding: Padding::default(),
        };
        let u = FrameUniforms::from_size(&size, shader_rgba(Rgb::new(0x1a, 0x1a, 0x1a), 0xff));
        assert_eq!(u.grid_columns, 80.0);
        assert_eq!(u.grid_rows, 30.0);
    }
}
