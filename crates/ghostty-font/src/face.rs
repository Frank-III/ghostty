//! Font face types (platform loading deferred). Port target: `src/font/face.zig`.

use crate::metrics::Metrics;

/// If a DPI cannot be calculated, use this fallback (72 on macOS, 96 elsewhere).
pub const fn default_dpi() -> u16 {
    #[cfg(target_os = "macos")]
    {
        72
    }
    #[cfg(not(target_os = "macos"))]
    {
        96
    }
}

/// Options for initializing a font face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    pub size: DesiredSize,
}

/// The desired size for loading a font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesiredSize {
    /// Desired size in points.
    pub points: f32,
    /// Horizontal DPI (defaults to [`default_dpi`]).
    pub xdpi: u16,
    /// Vertical DPI (defaults to [`default_dpi`]).
    pub ydpi: u16,
}

impl DesiredSize {
    pub const fn new(points: f32) -> Self {
        let dpi = default_dpi();
        Self {
            points,
            xdpi: dpi,
            ydpi: dpi,
        }
    }

    /// Converts points to pixels (1 point = 1/72 inch).
    pub fn pixels(self) -> f32 {
        (self.points * f32::from(self.ydpi)) / 72.0
    }
}

impl Default for DesiredSize {
    fn default() -> Self {
        Self::new(12.0)
    }
}

/// OpenType / CSS four-byte variation axis tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariationId(u32);

impl VariationId {
    pub const fn from_tag(tag: &[u8; 4]) -> Self {
        Self(u32::from_le_bytes(*tag))
    }

    pub const fn to_tag(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

/// A font variation setting (`font-variation-settings`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Variation {
    pub id: VariationId,
    pub value: f64,
}

/// The size and position of a glyph.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GlyphSize {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

/// Additional options for rendering glyphs (subset; constraint logic deferred).
#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub grid_metrics: Metrics,
    pub cell_width: Option<u8>,
    pub thicken: bool,
    pub thicken_strength: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            grid_metrics: Metrics::default(),
            cell_width: None,
            thicken: false,
            thicken_strength: 255,
        }
    }
}

#[cfg(target_os = "macos")]
pub mod platform {
    //! CoreText face loading (stub). Port target: `src/font/face/coretext.zig`.

    /// Placeholder until CoreText FFI is wired.
    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "CoreText face loading is not implemented in the Rust port yet"
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod freetype {
    //! FreeType face loading (stub). Port target: `src/font/face/freetype.zig`.

    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "FreeType face loading is not implemented in the Rust port yet"
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod web_canvas {
    //! Browser Canvas face (stub). Port target: `src/font/face/web_canvas.zig`.

    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "web_canvas face loading is not implemented in the Rust port yet"
        }
    }
}
