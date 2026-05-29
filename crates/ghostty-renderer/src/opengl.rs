//! OpenGL renderer stub (`src/renderer/OpenGL.zig` port target).

use crate::backend::Backend;
use crate::draw_backend::BackendRenderer;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;

/// OpenGL graphics API placeholder.
#[derive(Debug, Default, Clone, Copy)]
pub struct OpenGlGraphicsApi;

impl GraphicsApi for OpenGlGraphicsApi {
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

/// OpenGL-backed renderer (CPU draw prep until GL passes exist).
pub type OpenGlRenderer = BackendRenderer<OpenGlGraphicsApi>;

impl OpenGlRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(OpenGlGraphicsApi, Backend::OpenGl, size)
    }
}
