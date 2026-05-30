//! WebGL renderer stub (`src/renderer/WebGL.zig` port target).

use crate::backend::Backend;
use crate::draw_backend::BackendRenderer;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;

/// WebGL graphics API placeholder.
#[derive(Debug, Default, Clone)]
pub struct WebGlGraphicsApi {
    pub last_atlas_upload: Option<crate::atlas_texture::AtlasTexture>,
}

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

    fn upload_atlas_texture(
        &mut self,
        tex: &crate::atlas_texture::AtlasTexture,
    ) -> Result<(), GraphicsError> {
        self.last_atlas_upload = Some(tex.clone());
        Ok(())
    }
}

/// WebGL-backed renderer (CPU draw prep until WASM GL passes exist).
pub type WebGlRenderer = BackendRenderer<WebGlGraphicsApi>;

impl WebGlRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(WebGlGraphicsApi::default(), Backend::WebGl, size)
    }
}
