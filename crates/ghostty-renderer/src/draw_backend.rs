//! Shared CPU draw orchestration for GPU backend stubs.

use std::sync::Mutex;

use crate::atlas_texture::AtlasTexture;
use crate::backend::Backend;
use crate::cells::CellSnapshot;
use crate::damage::DamageState;
use crate::frame::{finish_draw_frame, prepare_draw_frame, FramePrep};
use crate::generic::{GenericRenderer, GraphicsApi, GraphicsError};
use crate::size::Size;

/// Host renderer holding API state and the last prepared frame.
pub struct BackendRenderer<A: GraphicsApi> {
    pub api: A,
    draw_mutex: Mutex<()>,
    pub size: Size,
    pub backend: Backend,
    pub frames_drawn: u64,
    last_prep: Option<FramePrep>,
    pub atlas_texture: Option<AtlasTexture>,
}

impl<A: GraphicsApi> BackendRenderer<A> {
    pub fn new(api: A, backend: Backend, size: Size) -> Result<Self, GraphicsError> {
        let mut renderer = Self {
            api,
            draw_mutex: Mutex::new(()),
            size,
            backend,
            frames_drawn: 0,
            last_prep: None,
            atlas_texture: None,
        };
        renderer.api.init_surface()?;
        Ok(renderer)
    }

    pub fn resize(&mut self, size: Size) -> Result<(), GraphicsError> {
        self.size = size;
        self.api.resize(size)
    }

    pub fn last_frame(&self) -> Option<&FramePrep> {
        self.last_prep.as_ref()
    }

    pub fn upload_atlas(&mut self, atlas: &ghostty_font::Atlas) -> Result<(), GraphicsError> {
        let tex = AtlasTexture::from_atlas(atlas);
        self.api.upload_atlas_texture(&tex)?;
        self.atlas_texture = Some(tex);
        Ok(())
    }

    pub fn draw_snapshot(
        &mut self,
        snapshot: &CellSnapshot,
        damage: &mut DamageState,
    ) -> Result<&FramePrep, GraphicsError> {
        let _lock = self
            .draw_mutex
            .lock()
            .map_err(|_| GraphicsError::DrawLockPoisoned)?;
        let prep = prepare_draw_frame(snapshot, damage);
        self.last_prep = Some(prep);
        self.frames_drawn = self.frames_drawn.saturating_add(1);
        finish_draw_frame(damage);
        Ok(self.last_prep.as_ref().expect("frame prep stored"))
    }
}

impl<A: GraphicsApi> GenericRenderer for BackendRenderer<A> {
    type Api = A;

    fn draw_mutex(&self) -> &Mutex<()> {
        &self.draw_mutex
    }

    fn size(&self) -> Size {
        self.size
    }

    fn draw_frame(&mut self) -> Result<(), GraphicsError> {
        Err(GraphicsError::NotImplemented(
            "draw_frame requires CellSnapshot; use draw_snapshot",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::CellSnapshot;
    use crate::generic::StubGraphicsApi;
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
    fn upload_atlas_stages_texture() {
        use ghostty_font::{Atlas, AtlasFormat};

        let mut atlas = Atlas::new(64, AtlasFormat::Grayscale);
        let region = atlas.reserve(8, 8).unwrap();
        atlas.write_grayscale(region, &[0; 64]);
        let mut renderer =
            BackendRenderer::new(StubGraphicsApi, Backend::Metal, test_size()).unwrap();
        renderer.upload_atlas(&atlas).unwrap();
        let tex = renderer.atlas_texture.as_ref().expect("atlas texture");
        assert_eq!(tex.size, 64);
        assert!(!tex.is_stale(&atlas));
    }

    #[test]
    fn metal_upload_forwards_to_api() {
        use ghostty_font::{Atlas, AtlasFormat};
        use crate::metal::MetalRenderer;

        let mut atlas = Atlas::new(32, AtlasFormat::Grayscale);
        let _ = atlas.reserve(4, 4).unwrap();
        let mut renderer = MetalRenderer::with_size(test_size()).unwrap();
        renderer.upload_atlas(&atlas).unwrap();
        assert!(renderer.api.last_atlas_upload.is_some());
    }

    #[test]
    fn draw_snapshot_records_frame() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, b'X' as u32);
        let mut damage = DamageState::default();
        let mut renderer =
            BackendRenderer::new(StubGraphicsApi, Backend::Metal, test_size()).unwrap();
        let prep = renderer.draw_snapshot(&snap, &mut damage).unwrap();
        assert_eq!(prep.populated_cells, 1);
        assert_eq!(renderer.frames_drawn, 1);
    }
}
