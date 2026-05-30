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
    //! CoreText face loading. Port target: `src/font/face/coretext.zig`.

    use core_graphics::base::CGFloat;
    use core_graphics::color_space::CGColorSpace;
    use core_graphics::context::CGContext;
    use core_graphics::font::CGGlyph;
    use core_graphics::geometry::{CGPoint, CGRect, CGSize};
    use core_graphics::image::CGImageAlphaInfo;
    use core_text::font;
    use core_text::font_descriptor::kCTFontOrientationHorizontal;

    use crate::atlas::Atlas;
    use crate::discovery::DiscoveredFont;
    use crate::glyph::{Glyph, GlyphRenderError};
    use crate::metrics::{calc, FaceMetrics, Metrics};
    use crate::RenderOptions;
    use crate::DesiredSize;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FaceLoadError {
        NotFound,
    }

    /// Metrics loaded from a CoreText font instance.
    #[derive(Debug)]
    pub struct LoadedFace {
        pub family: String,
        pub face_metrics: FaceMetrics,
        pub grid_metrics: Metrics,
        font: font::CTFont,
    }

    impl LoadedFace {
        pub fn open(discovered: &DiscoveredFont, size: DesiredSize) -> Result<Self, FaceLoadError> {
            let font = font::new_from_name(&discovered.family, f64::from(size.points))
                .map_err(|_| FaceLoadError::NotFound)?;
            let px = f64::from(size.pixels());
            let face_metrics = FaceMetrics {
                px_per_em: px,
                cell_width: max_visible_ascii_advance(&font),
                ascent: font.ascent(),
                descent: -font.descent().abs(),
                line_gap: font.leading(),
                cap_height: Some(font.cap_height()),
                ex_height: Some(font.x_height()),
                ..FaceMetrics::default()
            };
            Ok(Self {
                family: font.family_name(),
                grid_metrics: calc(face_metrics),
                face_metrics,
                font,
            })
        }

        pub fn render_glyph(
            &self,
            atlas: &mut Atlas,
            cp: u32,
            opts: &RenderOptions,
        ) -> Result<Glyph, GlyphRenderError> {
            let ch = char::from_u32(cp).ok_or(GlyphRenderError::InvalidCodepoint)?;
            if ch.is_control() {
                return Ok(Glyph::empty());
            }

            let mut unichars = [0u16; 2];
            let encoded_len = ch.encode_utf16(&mut unichars).len();
            let mut glyphs = [0u16; 2];
            let ok = unsafe {
                self.font.get_glyphs_for_characters(
                    unichars.as_ptr(),
                    glyphs.as_mut_ptr(),
                    encoded_len as isize,
                )
            };
            if !ok {
                return Err(GlyphRenderError::UnsupportedCodepoint);
            }

            let glyph = [glyphs[0] as CGGlyph];
            let bounds = self
                .font
                .get_bounding_rects_for_glyphs(kCTFontOrientationHorizontal, &glyph);

            let cell_w = opts.grid_metrics.cell_width.max(1);
            let cell_h = opts.grid_metrics.cell_height.max(1);
            let pad = 2u32;
            let px_width = cell_w.saturating_add(pad);
            let px_height = cell_h.saturating_add(pad);

            let color_space = CGColorSpace::create_device_gray();
            let alpha_only = CGImageAlphaInfo::CGImageAlphaOnly as u32;
            let mut context = CGContext::create_bitmap_context(
                None,
                px_width as usize,
                px_height as usize,
                8,
                px_width as usize,
                &color_space,
                alpha_only,
            );
            context.set_gray_fill_color(0.0, 0.0);
            context.fill_rect(CGRect::new(
                &CGPoint::new(0.0, 0.0),
                &CGSize::new(px_width as CGFloat, px_height as CGFloat),
            ));

            let strength = (f64::from(opts.thicken_strength) / 255.0).max(0.85);
            context.set_gray_fill_color(strength, 1.0);

            let x = (f64::from(px_width) - bounds.size.width) / 2.0 - bounds.origin.x;
            let y = f64::from(cell_h) - bounds.origin.y - bounds.size.height + f64::from(pad) / 2.0;
            context.save();
            context.translate(x, y);
            self.font
                .draw_glyphs(&glyph, &[CGPoint::new(0.0, 0.0)], context.clone());
            context.restore();
            context.flush();

            let region = atlas.reserve_or_grow(px_width, px_height)?;
            let ctx_stride = context.bytes_per_row();
            let data = context.data();
            let mut pixels = vec![0u8; (region.width * region.height) as usize];
            for row in 0..px_height as usize {
                let src = row * ctx_stride;
                let dst = row * px_width as usize;
                pixels[dst..dst + px_width as usize]
                    .copy_from_slice(&data[src..src + px_width as usize]);
            }
            atlas.write_grayscale(region, &pixels);

            Ok(Glyph {
                width: region.width,
                height: region.height,
                offset_x: 0,
                offset_y: cell_h as i32,
                atlas_x: region.x,
                atlas_y: region.y,
            })
        }
    }

    fn max_visible_ascii_advance(font: &font::CTFont) -> f64 {
        const COUNT: usize = 127 - 32;
        let unichars: Vec<u16> = (32u16..127).collect();
        let mut glyphs = vec![0u16; COUNT];
        let ok = unsafe {
            font.get_glyphs_for_characters(
                unichars.as_ptr(),
                glyphs.as_mut_ptr(),
                COUNT as isize,
            )
        };
        if !ok {
            return font.x_height();
        }
        let mut advances = vec![CGSize::new(0.0, 0.0); COUNT];
        let total = unsafe {
            font.get_advances_for_glyphs(
                kCTFontOrientationHorizontal,
                glyphs.as_ptr(),
                advances.as_mut_ptr(),
                COUNT as isize,
            )
        };
        if total <= 0.0 {
            return font.x_height();
        }
        advances.iter().map(|size| size.width).fold(0.0, f64::max)
    }

    /// Legacy stub type kept for callers not yet migrated to [`LoadedFace`].
    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "use platform::LoadedFace::open instead"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::descriptor_from_font_family;
        use crate::select_primary;

        #[test]
        fn loaded_face_metrics_for_monospace() {
            let descriptor = descriptor_from_font_family(Some("Menlo"), 12.0);
            let primary = select_primary(&descriptor).expect("discover");
            let face = LoadedFace::open(&primary, DesiredSize::new(12.0)).expect("open");
            assert!(!face.family.is_empty());
            assert!(face.grid_metrics.cell_width >= 1);
            assert!(face.grid_metrics.cell_height >= 1);
            assert!(face.face_metrics.ascent > 0.0);
            assert!(face.face_metrics.descent < 0.0);
        }

        #[test]
        fn render_glyph_rasterizes_ascii() {
            use crate::atlas::{Atlas, AtlasFormat};
            use crate::RenderOptions;

            let descriptor = descriptor_from_font_family(Some("Menlo"), 12.0);
            let primary = select_primary(&descriptor).expect("discover");
            let face = LoadedFace::open(&primary, DesiredSize::new(12.0)).expect("open");
            let mut atlas = Atlas::new(128, AtlasFormat::Grayscale);
            let opts = RenderOptions {
                grid_metrics: face.grid_metrics,
                ..RenderOptions::default()
            };
            let glyph = face
                .render_glyph(&mut atlas, b'A' as u32, &opts)
                .expect("render");
            assert!(glyph.width > 0);
            assert!(glyph.height > 0);
            assert!(glyph.atlas_x > 0);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub mod freetype {
    //! FreeType face loading. Port target: `src/font/face/freetype.zig`.

    use crate::atlas::Atlas;
    use crate::discovery::DiscoveredFont;
    use crate::glyph::{Glyph, GlyphRenderError};
    use crate::metrics::{calc, FaceMetrics, Metrics};
    use crate::RenderOptions;
    use crate::DesiredSize;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FaceLoadError {
        NotFound,
        Library,
        Face,
    }

    #[derive(Debug)]
    pub struct LoadedFace {
        pub family: String,
        pub face_metrics: FaceMetrics,
        pub grid_metrics: Metrics,
        _library: freetype::Library,
        face: freetype::Face,
    }

    impl LoadedFace {
        pub fn open(discovered: &DiscoveredFont, size: DesiredSize) -> Result<Self, FaceLoadError> {
            let path = discovered.path.as_deref().ok_or(FaceLoadError::NotFound)?;
            let library = freetype::Library::init().map_err(|_| FaceLoadError::Library)?;
            let face = library
                .new_face(path, discovered.face_index)
                .map_err(|_| FaceLoadError::Face)?;
            face.set_char_size(
                0,
                (f64::from(size.points) * 64.0) as isize,
                size.xdpi as u32,
                size.ydpi as u32,
            )
            .map_err(|_| FaceLoadError::Face)?;
            let metrics = face.size_metrics().ok_or(FaceLoadError::Face)?;
            let scale = 1.0 / 64.0;
            let ascent = f64::from(metrics.ascender) * scale;
            let descent = f64::from(metrics.descender) * scale;
            let line_gap = f64::from(
                metrics
                    .height
                    .saturating_sub(metrics.ascender - metrics.descender),
            ) * scale;
            let cell_width =
                max_ascii_advance(&face).unwrap_or(f64::from(metrics.max_advance) * scale);
            let face_metrics = FaceMetrics {
                px_per_em: f64::from(size.pixels()),
                cell_width,
                ascent,
                descent,
                line_gap,
                ..FaceMetrics::default()
            };
            Ok(Self {
                family: discovered.family.clone(),
                grid_metrics: calc(face_metrics),
                face_metrics,
                _library: library,
                face,
            })
        }

        pub fn render_glyph(
            &self,
            atlas: &mut Atlas,
            cp: u32,
            opts: &RenderOptions,
        ) -> Result<Glyph, GlyphRenderError> {
            let ch = char::from_u32(cp).ok_or(GlyphRenderError::InvalidCodepoint)?;
            if ch.is_control() {
                return Ok(Glyph::empty());
            }

            let cell_w = opts.grid_metrics.cell_width.max(1);
            let cell_h = opts.grid_metrics.cell_height.max(1);

            let glyph_index = self.face.get_char_index(ch as u32);
            if glyph_index == 0 && ch != ' ' {
                return Err(GlyphRenderError::UnsupportedCodepoint);
            }
            self.face
                .load_glyph(glyph_index, freetype::face::LoadFlag::RENDER)
                .map_err(|_| GlyphRenderError::Platform)?;
            let glyph = self.face.glyph();
            let bitmap = glyph.bitmap();
            let width = bitmap.width().max(0) as u32;
            let height = bitmap.rows().max(0) as u32;

            if width == 0 || height == 0 {
                let region = atlas.reserve_or_grow(cell_w, cell_h)?;
                let pixels = vec![0u8; (region.width * region.height) as usize];
                atlas.write_grayscale(region, &pixels);
                return Ok(Glyph {
                    width: region.width,
                    height: region.height,
                    offset_x: 0,
                    offset_y: cell_h as i32,
                    atlas_x: region.x,
                    atlas_y: region.y,
                });
            }

            let region = atlas.reserve_or_grow(width, height)?;
            let buffer = bitmap.buffer();
            if matches!(bitmap.pixel_mode(), freetype::bitmap::PixelMode::Gray) {
                atlas.write_grayscale(region, buffer);
            } else {
                let pixels = vec![0xff; (region.width * region.height) as usize];
                atlas.write_grayscale(region, &pixels);
            }

            Ok(Glyph {
                width: region.width,
                height: region.height,
                offset_x: glyph.bitmap_left(),
                offset_y: cell_h as i32 - glyph.bitmap_top(),
                atlas_x: region.x,
                atlas_y: region.y,
            })
        }
    }

    fn max_ascii_advance(face: &freetype::Face) -> Option<f64> {
        let mut max = 0.0f64;
        for ch in 32u8..127 {
            if face.load_char(ch as usize, freetype::face::LoadFlag::DEFAULT).is_err() {
                continue;
            }
            let advance = face.glyph().advance().x as f64 / 64.0;
            max = max.max(advance);
        }
        if max > 0.0 { Some(max) } else { None }
    }

    /// Legacy stub type kept for callers not yet migrated to [`LoadedFace`].
    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "use freetype::LoadedFace::open instead"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::descriptor_from_font_family;
        use crate::select_primary;

        #[test]
        fn loaded_face_metrics_for_monospace() {
            let descriptor = descriptor_from_font_family(None, 12.0);
            let primary = select_primary(&descriptor).expect("discover");
            let face = LoadedFace::open(&primary, DesiredSize::new(12.0)).expect("open");
            assert!(!face.family.is_empty());
            assert!(face.grid_metrics.cell_width >= 1);
            assert!(face.grid_metrics.cell_height >= 1);
        }
    }
}

#[cfg(any(not(unix), target_os = "macos", target_arch = "wasm32"))]
pub mod freetype {
    //! FreeType face loading (stub). Port target: `src/font/face/freetype.zig`.

    #[derive(Debug)]
    pub struct Face;

    impl Face {
        pub const fn unavailable() -> &'static str {
            "FreeType face loading is not implemented on this target yet"
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
