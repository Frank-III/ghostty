//! Texture atlas rectangle bin pack (`src/font/Atlas.zig` subset).

use std::sync::atomic::{AtomicUsize, Ordering};

/// Pixel format for atlas texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AtlasFormat {
    #[default]
    Grayscale,
    Bgr,
    Bgra,
}

impl AtlasFormat {
    pub const fn depth(self) -> usize {
        match self {
            Self::Grayscale => 1,
            Self::Bgr => 3,
            Self::Bgra => 4,
        }
    }
}

/// Reserved sub-rectangle within the atlas texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasError {
    Full,
    ShrinkNotSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    x: u32,
    y: u32,
    width: u32,
}

/// Square GPU texture atlas with shelf-style rectangle packing.
#[derive(Debug)]
pub struct Atlas {
    data: Vec<u8>,
    size: u32,
    format: AtlasFormat,
    nodes: Vec<Node>,
    modified: AtomicUsize,
    resized: AtomicUsize,
}

impl Atlas {
    pub fn new(size: u32, format: AtlasFormat) -> Self {
        let depth = format.depth();
        let mut atlas = Self {
            data: vec![0; (size as usize) * (size as usize) * depth],
            size,
            format,
            nodes: Vec::with_capacity(64),
            modified: AtomicUsize::new(0),
            resized: AtomicUsize::new(0),
        };
        atlas.clear();
        atlas
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn format(&self) -> AtlasFormat {
        self.format
    }

    pub fn modified_generation(&self) -> usize {
        self.modified.load(Ordering::Relaxed)
    }

    pub fn resized_generation(&self) -> usize {
        self.resized.load(Ordering::Relaxed)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
        self.nodes.clear();
        // 1px border to avoid texture sampling artifacts (matches `Atlas.zig`).
        self.nodes.push(Node {
            x: 1,
            y: 1,
            width: self.size.saturating_sub(2),
        });
        self.bump_modified();
    }

    /// Grow the texture, preserving packed regions (`Atlas.zig` `grow`).
    pub fn grow(&mut self, size_new: u32) -> Result<(), AtlasError> {
        if size_new < self.size {
            return Err(AtlasError::ShrinkNotSupported);
        }
        if size_new == self.size {
            return Ok(());
        }

        let depth = self.format.depth();
        let size_old = self.size;
        let stride_old = size_old as usize * depth;
        let stride_new = size_new as usize * depth;
        let mut data_new = vec![0u8; (size_new as usize) * (size_new as usize) * depth];

        // Copy interior rows, skipping the 1px top/bottom border rows.
        for row in 1..size_old.saturating_sub(1) {
            let src = row as usize * stride_old;
            let dst = row as usize * stride_new;
            data_new[dst..dst + stride_old]
                .copy_from_slice(&self.data[src..src + stride_old]);
        }

        self.data = data_new;
        self.size = size_new;
        self.nodes.push(Node {
            x: size_old.saturating_sub(1),
            y: 1,
            width: size_new.saturating_sub(size_old),
        });
        self.modified.fetch_add(1, Ordering::Relaxed);
        self.resized.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Reserve a region, doubling atlas size until space is available.
    pub fn reserve_or_grow(&mut self, width: u32, height: u32) -> Result<AtlasRegion, AtlasError> {
        if let Ok(region) = self.reserve(width, height) {
            return Ok(region);
        }
        let size_new = self.size.saturating_mul(2).max(64);
        self.grow(size_new)?;
        self.reserve(width, height)
    }

    pub fn reserve(&mut self, width: u32, height: u32) -> Result<AtlasRegion, AtlasError> {
        if width == 0 && height == 0 {
            return Ok(AtlasRegion {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            });
        }

        let mut best_idx = None;
        let mut best_height = u32::MAX;
        let mut best_width = u32::MAX;
        let mut region = AtlasRegion {
            x: 0,
            y: 0,
            width,
            height,
        };

        for (i, _) in self.nodes.iter().enumerate() {
            let Some(y) = self.fit(i, width, height) else {
                continue;
            };
            let node = self.nodes[i];
            if y + height < best_height
                || (y + height == best_height && node.width > 0 && node.width < best_width)
            {
                best_idx = Some(i);
                best_width = node.width;
                best_height = y + height;
                region.x = node.x;
                region.y = y;
            }
        }

        let best_idx = best_idx.ok_or(AtlasError::Full)?;
        self.nodes.insert(
            best_idx,
            Node {
                x: region.x,
                y: region.y + height,
                width,
            },
        );

        let i = best_idx + 1;
        while i < self.nodes.len() {
            let prev = self.nodes[i - 1];
            let node = &mut self.nodes[i];
            if node.x < prev.x + prev.width {
                let shrink = prev.x + prev.width - node.x;
                node.x += shrink;
                node.width = node.width.saturating_sub(shrink);
                if node.width == 0 {
                    self.nodes.remove(i);
                    continue;
                }
            }
            break;
        }
        self.merge_nodes();

        self.bump_modified();
        Ok(region)
    }

    pub fn write_grayscale(&mut self, region: AtlasRegion, pixels: &[u8]) {
        let expected = (region.width as usize) * (region.height as usize);
        if pixels.len() != expected {
            return;
        }
        let stride = self.size as usize;
        for row in 0..region.height as usize {
            let dst_row = (region.y as usize + row) * stride + region.x as usize;
            let src_row = row * region.width as usize;
            let end = src_row + region.width as usize;
            self.data[dst_row..dst_row + region.width as usize]
                .copy_from_slice(&pixels[src_row..end]);
        }
        self.bump_modified();
    }

    fn fit(&self, idx: usize, width: u32, height: u32) -> Option<u32> {
        let node = self.nodes.get(idx)?;
        if node.x + width > self.size.saturating_sub(1) {
            return None;
        }
        let mut y = node.y;
        let mut i = idx;
        let mut width_left = width;
        while width_left > 0 {
            let n = self.nodes.get(i)?;
            if n.y > y {
                y = n.y;
            }
            if y + height > self.size.saturating_sub(1) {
                return None;
            }
            width_left = width_left.saturating_sub(n.width);
            i += 1;
        }
        Some(y)
    }

    fn merge_nodes(&mut self) {
        let mut i = 0;
        while i + 1 < self.nodes.len() {
            if self.nodes[i].y == self.nodes[i + 1].y {
                self.nodes[i].width += self.nodes[i + 1].width;
                self.nodes.remove(i + 1);
                continue;
            }
            i += 1;
        }
    }

    fn bump_modified(&self) {
        self.modified.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_two_regions() {
        let mut atlas = Atlas::new(64, AtlasFormat::Grayscale);
        let a = atlas.reserve(8, 8).unwrap();
        let b = atlas.reserve(8, 8).unwrap();
        assert_ne!((a.x, a.y), (b.x, b.y));
    }

    #[test]
    fn write_grayscale_round_trip() {
        let mut atlas = Atlas::new(16, AtlasFormat::Grayscale);
        let region = atlas.reserve(2, 2).unwrap();
        atlas.write_grayscale(region, &[1, 2, 3, 4]);
        let idx = region.y as usize * 16 + region.x as usize;
        assert_eq!(atlas.data[idx], 1);
    }

    #[test]
    fn grow_doubles_and_preserves_pixels() {
        let mut atlas = Atlas::new(16, AtlasFormat::Grayscale);
        let region = atlas.reserve(2, 2).unwrap();
        atlas.write_grayscale(region, &[9, 9, 9, 9]);
        let old_resized = atlas.resized_generation();
        atlas.grow(32).unwrap();
        assert_eq!(atlas.size(), 32);
        assert!(atlas.resized_generation() > old_resized);
        let idx = region.y as usize * 32 + region.x as usize;
        assert_eq!(atlas.data()[idx], 9);
    }

    #[test]
    fn reserve_or_grow_expands_when_full() {
        let mut atlas = Atlas::new(16, AtlasFormat::Grayscale);
        while atlas.reserve(8, 8).is_ok() {}
        let old_size = atlas.size();
        let region = atlas.reserve_or_grow(8, 8).expect("grow and reserve");
        assert!(atlas.size() > old_size);
        assert_eq!(region.width, 8);
    }
}
