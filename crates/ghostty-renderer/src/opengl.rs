//! OpenGL renderer stub (`src/renderer/OpenGL.zig` port target).

use crate::backend::Backend;
use crate::draw_backend::BackendRenderer;
use crate::generic::{GraphicsApi, GraphicsError};
use crate::size::Size;

/// OpenGL graphics API placeholder.
#[derive(Debug, Default, Clone)]
pub struct OpenGlGraphicsApi {
    pub last_atlas_upload: Option<crate::atlas_texture::AtlasTexture>,
}

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

    fn upload_atlas_texture(&mut self, tex: &crate::atlas_texture::AtlasTexture) -> Result<(), GraphicsError> {
        self.last_atlas_upload = Some(tex.clone());
        Ok(())
    }
}

/// OpenGL-backed renderer (CPU draw prep until GL passes exist).
pub type OpenGlRenderer = BackendRenderer<OpenGlGraphicsApi>;

impl OpenGlRenderer {
    pub fn with_size(size: Size) -> Result<Self, GraphicsError> {
        BackendRenderer::new(OpenGlGraphicsApi::default(), Backend::OpenGl, size)
    }
}
