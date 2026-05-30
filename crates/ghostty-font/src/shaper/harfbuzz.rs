//! HarfBuzz-compatible shaping via rustybuzz. Port target: `src/font/shaper/harfbuzz.zig`.

use std::path::Path;

use rustybuzz::UnicodeBuffer;
use ttf_parser::Face;

/// One shaped glyph with terminal-grid placement hints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub cluster_start: usize,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
}

/// HarfBuzz-backed shaping session (pure Rust via rustybuzz).
pub struct HarfBuzzShaper {
    face: rustybuzz::Face<'static>,
    enabled: bool,
}

impl std::fmt::Debug for HarfBuzzShaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarfBuzzShaper")
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

impl HarfBuzzShaper {
    /// Load a font file for shaping. Returns `None` when the path is missing or invalid.
    pub fn open(path: impl AsRef<Path>) -> Option<Self> {
        let bytes: Box<[u8]> = std::fs::read(path.as_ref()).ok()?.into();
        let static_bytes: &'static [u8] = Box::leak(bytes);
        let ttf = Face::parse(static_bytes, 0).ok()?;
        let face = rustybuzz::Face::from_face(ttf);
        Some(Self {
            face,
            enabled: true,
        })
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Shape UTF-32 codepoints into positioned glyphs.
    pub fn shape(&self, codepoints: &[u32], font_size: f32) -> Vec<ShapedGlyph> {
        if !self.enabled || codepoints.is_empty() {
            return Vec::new();
        }
        let mut buffer = UnicodeBuffer::new();
        for &cp in codepoints {
            if let Some(ch) = char::from_u32(cp) {
                buffer.add(ch, cp);
            }
        }
        let output = rustybuzz::shape(&self.face, &[], buffer);
        let scale = font_size / self.face.units_per_em() as f32;
        output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions().iter())
            .map(|(info, pos)| ShapedGlyph {
                glyph_id: info.glyph_id,
                cluster_start: info.cluster as usize,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
                x_advance: pos.x_advance as f32 * scale,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_ascii_from_system_font() {
        let path = if cfg!(target_os = "macos") {
            "/System/Library/Fonts/Menlo.ttc"
        } else {
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
        };
        if !Path::new(path).exists() {
            return;
        }
        let shaper = HarfBuzzShaper::open(path).expect("font");
        let shaped = shaper.shape(&[b'a' as u32, b'b' as u32, b'c' as u32], 12.0);
        assert_eq!(shaped.len(), 3);
    }
}
