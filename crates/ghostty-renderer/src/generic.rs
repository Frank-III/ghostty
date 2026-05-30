//! Generic renderer trait skeleton over a graphics API.
//!
//! Port target: `src/renderer/generic.zig` (full `drawFrame` / cell rebuild deferred).
//!
//! GTK note: `apprt.gtk.App.must_draw_from_app_thread` forces redraw messages instead of
//! calling `draw_frame` from the renderer thread (`src/renderer/Thread.zig`). Future Rust
//! glue must preserve that constraint on Linux.

use crate::cell::{CellBgDraw, CellText};
use crate::draw_pass::{draw_background_pass_stub, draw_text_pass_stub, DrawPassStats};
use crate::size::Size;

/// Metal / OpenGL / WebGL surface abstraction (`Renderer(GraphicsAPI)` hierarchy).
///
/// Concrete backends implement buffer upload, pipelines, and render passes. This
/// trait only documents the expected shape; GPU types are not wired yet.
pub trait GraphicsApi: Sized {
    type Target;
    type Frame;
    type RenderPass;
    type Pipeline;
    type Buffer;
    type Texture;
    type Sampler;

    fn init_surface(&mut self) -> Result<(), GraphicsError>;
    fn resize(&mut self, size: Size) -> Result<(), GraphicsError>;

    fn upload_atlas_texture(
        &mut self,
        _tex: &crate::atlas_texture::AtlasTexture,
    ) -> Result<(), GraphicsError> {
        Ok(())
    }

    /// Background cell quads for the current frame (stub validates grid bounds).
    fn draw_background_pass(
        &mut self,
        size: &Size,
        cells: &[CellBgDraw],
    ) -> Result<(), GraphicsError> {
        draw_background_pass_stub(size, cells)
    }

    /// Foreground glyph instances (stub validates grid bounds).
    fn draw_text_pass(&mut self, size: &Size, cells: &[CellText]) -> Result<(), GraphicsError> {
        draw_text_pass_stub(size, cells)
    }

    /// Last draw pass statistics when the API records them (optional).
    fn last_draw_pass(&self) -> Option<DrawPassStats> {
        None
    }
}

/// Frame draw entry point for a generic renderer.
pub trait GenericRenderer: Sized {
    type Api: GraphicsApi;

    /// Must be held while mutating draw state or inside `draw_frame` (Zig `draw_mutex`).
    fn draw_mutex(&self) -> &std::sync::Mutex<()>;

    fn size(&self) -> Size;

    /// Rebuild GPU cell buffers from terminal state (not implemented).
    fn rebuild_cells(&mut self) -> Result<(), GraphicsError> {
        Err(GraphicsError::NotImplemented("rebuild_cells"))
    }

    /// Issue draw commands for the current frame (not implemented).
    fn draw_frame(&mut self) -> Result<(), GraphicsError> {
        Err(GraphicsError::NotImplemented("draw_frame"))
    }
}

/// Placeholder graphics error until backends land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsError {
    NotImplemented(&'static str),
    DrawLockPoisoned,
    InvalidDrawInstance,
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsError::NotImplemented(name) => write!(f, "renderer not implemented: {name}"),
            GraphicsError::DrawLockPoisoned => write!(f, "renderer draw mutex poisoned"),
            GraphicsError::InvalidDrawInstance => write!(f, "draw instance outside grid"),
        }
    }
}

impl std::error::Error for GraphicsError {}

/// Stub generic renderer holding layout state only.
pub struct GenericRendererStub {
    draw_mutex: std::sync::Mutex<()>,
    pub size: Size,
}

impl GenericRendererStub {
    pub fn new(size: Size) -> Self {
        Self {
            draw_mutex: std::sync::Mutex::new(()),
            size,
        }
    }
}

impl GenericRenderer for GenericRendererStub {
    type Api = StubGraphicsApi;

    fn draw_mutex(&self) -> &std::sync::Mutex<()> {
        &self.draw_mutex
    }

    fn size(&self) -> Size {
        self.size
    }
}

/// No-op graphics API for compile-time trait checking.
pub struct StubGraphicsApi;

impl GraphicsApi for StubGraphicsApi {
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
        _tex: &crate::atlas_texture::AtlasTexture,
    ) -> Result<(), GraphicsError> {
        Ok(())
    }
}
