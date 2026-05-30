//! Platform font session for metrics + glyph rasterization.

use crate::discovery::DiscoveredFont;
use crate::glyph::{Glyph, GlyphRenderError};
use crate::metrics::Metrics;
use crate::{Atlas, DesiredSize, RenderOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSessionError {
    NotFound,
    Platform,
}

/// Loaded primary font used for grid metrics and atlas rasterization.
#[derive(Debug)]
pub enum FontSession {
    #[cfg(target_os = "macos")]
    CoreText(crate::face::platform::LoadedFace),
    #[cfg(all(unix, not(target_os = "macos")))]
    FreeType(crate::face::freetype::LoadedFace),
}

impl FontSession {
    pub fn open(discovered: &DiscoveredFont, size: DesiredSize) -> Result<Self, FontSessionError> {
        #[cfg(target_os = "macos")]
        {
            return crate::face::platform::LoadedFace::open(discovered, size)
                .map(Self::CoreText)
                .map_err(|_| FontSessionError::NotFound);
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            return crate::face::freetype::LoadedFace::open(discovered, size)
                .map(Self::FreeType)
                .map_err(|err| match err {
                    crate::face::freetype::FaceLoadError::NotFound => FontSessionError::NotFound,
                    _ => FontSessionError::Platform,
                });
        }
        #[cfg(not(any(target_os = "macos", all(unix, not(target_os = "macos")))))]
        {
            let _ = (discovered, size);
            Err(FontSessionError::Platform)
        }
    }

    pub fn grid_metrics(&self) -> Metrics {
        match self {
            #[cfg(target_os = "macos")]
            Self::CoreText(face) => face.grid_metrics,
            #[cfg(all(unix, not(target_os = "macos")))]
            Self::FreeType(face) => face.grid_metrics,
        }
    }

    pub fn render_glyph(
        &self,
        atlas: &mut Atlas,
        cp: u32,
        opts: &RenderOptions,
    ) -> Result<Glyph, GlyphRenderError> {
        match self {
            #[cfg(target_os = "macos")]
            Self::CoreText(face) => face.render_glyph(atlas, cp, opts),
            #[cfg(all(unix, not(target_os = "macos")))]
            Self::FreeType(face) => face.render_glyph(atlas, cp, opts),
        }
    }
}
