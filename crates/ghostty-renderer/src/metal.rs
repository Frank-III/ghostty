//! Metal renderer stub (`src/renderer/Metal.zig` port target).

use crate::backend::Backend;
use crate::draw_backend::BackendRenderer;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;

/// Metal graphics API placeholder until pipelines and shaders land.
#[derive(Debug, Default, Clone, Copy)]
pub struct MetalGraphicsApi;

impl GraphicsApi for MetalGraphicsApi {
    type Target = ();
    type Frame = ();
    type RenderPass = ();
    type Pipeline = ();
    type Buffer = ();
    type Texture = ();
    type Sampler = ();

    fn init_surface(&mut self) -> Result<(), GraphicsError> {
        Ok(())
    }

    fn resize(&mut self, _size: Size) -> Result<(), GraphicsError> {
        Ok(())
    }
}

/// Metal-backed renderer (CPU draw prep until GPU passes exist).
pub type MetalRenderer = BackendRenderer<MetalGraphicsApi>;

impl MetalRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(MetalGraphicsApi, Backend::Metal, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::CellSnapshot;
    use crate::damage::DamageState;
    use crate::size::{CellSize, GridSize, Padding, ScreenSize, Size};

    fn test_size() -> Size {
        Size {
            screen: ScreenSize {
                width: 640,
                height: 480,
            },
            cell: CellSize {
                width: 8,
                height: 16,
            },
            padding: Padding::default(),
        }
    }

    #[test]
    fn metal_draw_snapshot() {
        let grid = GridSize {
            columns: 4,
            rows: 2,
        };
        let snap = CellSnapshot::empty(grid);
        let mut damage = DamageState::default();
        let mut renderer = MetalRenderer::with_size(test_size()).unwrap();
        renderer.draw_snapshot(&snap, &mut damage).unwrap();
        assert_eq!(renderer.backend, Backend::Metal);
    }
}
