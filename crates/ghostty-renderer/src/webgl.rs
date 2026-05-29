//! WebGL renderer stub (`src/renderer/WebGL.zig` port target).

use crate::backend::Backend;
use crate::draw_backend::BackendRenderer;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;

/// WebGL graphics API placeholder.
#[derive(Debug, Default, Clone, Copy)]
pub struct WebGlGraphicsApi;

impl GraphicsApi for WebGlGraphicsApi {
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

/// WebGL-backed renderer (CPU draw prep until WASM GL passes exist).
pub type WebGlRenderer = BackendRenderer<WebGlGraphicsApi>;

impl WebGlRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(WebGlGraphicsApi, Backend::WebGl, size)
    }
}
