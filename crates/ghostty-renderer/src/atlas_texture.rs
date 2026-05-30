//! GPU-side atlas texture staging (`src/font/Atlas.zig` upload path subset).

use ghostty_font::{Atlas, AtlasFormat};

/// CPU→GPU staging buffer for a font atlas texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtlasTexture {
    pub size: u32,
    pub format: AtlasFormat,
    pub pixels: Vec<u8>,
    pub modified_generation: usize,
}

impl AtlasTexture {
    pub fn from_atlas(atlas: &Atlas) -> Self {
        Self {
            size: atlas.size(),
            format: atlas.format(),
            pixels: atlas.data().to_vec(),
            modified_generation: atlas.modified_generation(),
        }
    }

    pub fn depth(&self) -> usize {
        self.format.depth()
    }

    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }

    pub fn is_stale(&self, atlas: &Atlas) -> bool {
        self.modified_generation != atlas.modified_generation()
    }
}

#[cfg(test)]
mod tests {
    use ghostty_font::{Atlas, AtlasFormat};

    use super::*;

    #[test]
    fn from_atlas_dimensions() {
        let mut atlas = Atlas::new(32, AtlasFormat::Grayscale);
        let region = atlas.reserve(4, 4).unwrap();
        atlas.write_grayscale(region, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let tex = AtlasTexture::from_atlas(&atlas);
        assert_eq!(tex.size, 32);
        assert_eq!(tex.byte_len(), 32 * 32);
        assert_eq!(tex.modified_generation, atlas.modified_generation());
        let idx = (region.y * tex.size + region.x) as usize;
        assert_eq!(tex.pixels[idx], 1);
    }
}
