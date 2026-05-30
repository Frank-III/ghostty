mod gpu;

pub use gpu::MetalGpuState;

use crate::backend::Backend;
use crate::cell::{CellBgDraw, CellText};
use crate::color::Rgb;
use crate::draw_backend::BackendRenderer;
use crate::draw_pass::DrawPassStats;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;
use crate::uniforms::{CursorUniforms, FrameUniforms};

/// Metal graphics API with GPU buffer encoding on macOS.
#[derive(Debug)]
pub struct MetalGraphicsApi {
    pub last_atlas_upload: Option<crate::atlas_texture::AtlasTexture>,
    pub gpu: MetalGpuState,
    #[cfg(target_os = "macos")]
    device: Option<metal::Device>,
    last_stats: Option<DrawPassStats>,
}

impl Default for MetalGraphicsApi {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let device = gpu::init_device().map(|(d, _)| d);
        Self {
            last_atlas_upload: None,
            gpu: MetalGpuState::default(),
            #[cfg(target_os = "macos")]
            device,
            last_stats: None,
        }
    }
}

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

    fn upload_atlas_texture(
        &mut self,
        tex: &crate::atlas_texture::AtlasTexture,
    ) -> Result<(), GraphicsError> {
        self.last_atlas_upload = Some(tex.clone());
        Ok(())
    }

    fn draw_background_pass(
        &mut self,
        size: &Size,
        cells: &[CellBgDraw],
        uniforms: &FrameUniforms,
    ) -> Result<(), GraphicsError> {
        crate::draw_pass::draw_background_pass_stub(size, cells)?;
        let default_bg = Rgb::new(
            uniforms.bg_color[0],
            uniforms.bg_color[1],
            uniforms.bg_color[2],
        );
        #[cfg(target_os = "macos")]
        if let Some(device) = self.device.as_ref() {
            self.gpu = gpu::encode_frame(device, size, cells, &[], uniforms, None, default_bg);
        }
        let _ = default_bg;
        Ok(())
    }

    fn draw_text_pass(
        &mut self,
        size: &Size,
        cells: &[CellText],
        cursor: Option<&CursorUniforms>,
    ) -> Result<(), GraphicsError> {
        crate::draw_pass::draw_text_pass_stub(size, cells)?;
        #[cfg(target_os = "macos")]
        if let Some(device) = self.device.as_ref() {
            let uniforms = FrameUniforms::from_size(size, [0, 0, 0, 0xff]);
            self.gpu = gpu::encode_frame(
                device,
                size,
                &[],
                cells,
                &uniforms,
                cursor,
                Rgb::new(0, 0, 0),
            );
        }
        self.last_stats = Some(DrawPassStats {
            bg_instances: 0,
            text_instances: cells.len(),
            cursor_instances: usize::from(cursor.is_some()),
        });
        Ok(())
    }

    fn last_draw_pass(&self) -> Option<DrawPassStats> {
        self.last_stats
    }
}

/// Metal-backed renderer.
pub type MetalRenderer = BackendRenderer<MetalGraphicsApi>;

impl MetalRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(MetalGraphicsApi::default(), Backend::Metal, size)
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
