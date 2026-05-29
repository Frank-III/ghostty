//! Platform renderer factory.

use crate::backend::Backend;
use crate::cells::CellSnapshot;
use crate::damage::DamageState;
use crate::frame::FramePrep;
use crate::generic::GraphicsError;
use crate::metal::MetalRenderer;
use crate::opengl::OpenGlRenderer;
use crate::size::Size;
use crate::webgl::WebGlRenderer;

/// Platform-selected renderer backend.
pub enum HostRenderer {
    Metal(MetalRenderer),
    OpenGl(OpenGlRenderer),
    WebGl(WebGlRenderer),
}

impl HostRenderer {
    pub fn for_host(size: Size) -> Result<Self, GraphicsError> {
        Self::for_backend(Backend::default_for_host(), size)
    }

    pub fn for_backend(backend: Backend, size: Size) -> Result<Self, GraphicsError> {
        Ok(match backend {
            Backend::Metal => Self::Metal(MetalRenderer::with_size(size)?),
            Backend::OpenGl => Self::OpenGl(OpenGlRenderer::with_size(size)?),
            Backend::WebGl => Self::WebGl(WebGlRenderer::with_size(size)?),
        })
    }

    pub fn backend(&self) -> Backend {
        match self {
            Self::Metal(r) => r.backend,
            Self::OpenGl(r) => r.backend,
            Self::WebGl(r) => r.backend,
        }
    }

    pub fn draw_snapshot(
        &mut self,
        snapshot: &CellSnapshot,
        damage: &mut DamageState,
    ) -> Result<&FramePrep, GraphicsError> {
        match self {
            Self::Metal(r) => r.draw_snapshot(snapshot, damage),
            Self::OpenGl(r) => r.draw_snapshot(snapshot, damage),
            Self::WebGl(r) => r.draw_snapshot(snapshot, damage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::{CellSize, Padding, ScreenSize, Size};

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
    fn host_renderer_matches_platform_default() {
        let renderer = HostRenderer::for_host(test_size()).unwrap();
        assert_eq!(renderer.backend(), Backend::default_for_host());
    }

    #[test]
    fn each_backend_constructs() {
        let size = test_size();
        for backend in [Backend::Metal, Backend::OpenGl, Backend::WebGl] {
            let r = HostRenderer::for_backend(backend, size).unwrap();
            assert_eq!(r.backend(), backend);
        }
    }
}
