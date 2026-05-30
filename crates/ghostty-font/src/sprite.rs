//! Non-Unicode sprite glyphs (cursor shapes). Port target: `src/font/sprite/draw/special.zig`.

use crate::{Atlas, AtlasRegion, RenderOptions};
use crate::{Glyph, GlyphRenderError};

/// Cursor sprite variants drawn into the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorSprite {
    Block,
    BlockHollow,
    Bar,
    Underline,
}

impl CursorSprite {
    pub fn from_visual_style(raw: u8) -> Self {
        match raw {
            0 => Self::Bar,
            2 => Self::Underline,
            3 => Self::BlockHollow,
            _ => Self::Block,
        }
    }
}

/// Rasterize a cursor sprite into the atlas and return placement metadata.
pub fn render_cursor_sprite(
    atlas: &mut Atlas,
    sprite: CursorSprite,
    opts: &RenderOptions,
) -> Result<Glyph, GlyphRenderError> {
    let (cell_w, cell_h) = crate::face::render_cell_size(opts);
    let metrics = opts.grid_metrics;
    let region = atlas
        .reserve(cell_w, cell_h)
        .map_err(GlyphRenderError::from)?;
    let thickness = metrics.cursor_thickness.max(1);
    let cell_w = cell_w as usize;
    let cell_h = cell_h as usize;
    let mut pixels = vec![0u8; cell_w * cell_h];
    match sprite {
        CursorSprite::Block => fill_rect(
            &mut pixels,
            cell_w,
            cell_h,
            0,
            0,
            cell_w as u32,
            cell_h as u32,
        ),
        CursorSprite::BlockHollow => {
            fill_rect(
                &mut pixels,
                cell_w,
                cell_h,
                0,
                0,
                cell_w as u32,
                cell_h as u32,
            );
            hollow_rect(
                &mut pixels,
                cell_w,
                cell_h,
                thickness,
                thickness,
                cell_w as u32,
                cell_h as u32,
            );
        }
        CursorSprite::Bar => {
            let x = thickness.saturating_sub(1) / 2;
            fill_rect(&mut pixels, cell_w, cell_h, x, 0, thickness, cell_h as u32);
        }
        CursorSprite::Underline => {
            let y = metrics
                .underline_position
                .min(cell_h as u32)
                .saturating_sub(thickness);
            fill_rect(&mut pixels, cell_w, cell_h, 0, y, cell_w as u32, thickness);
        }
    }
    atlas.write_grayscale(region, &pixels);
    Ok(glyph_from_region(region, cell_w as u32, cell_h as u32))
}

fn glyph_from_region(region: AtlasRegion, cell_w: u32, cell_h: u32) -> Glyph {
    Glyph {
        atlas_x: region.x,
        atlas_y: region.y,
        width: cell_w,
        height: cell_h,
        offset_x: 0,
        offset_y: 0,
    }
}

fn fill_rect(pixels: &mut [u8], stride: usize, height: usize, x: u32, y: u32, w: u32, h: u32) {
    for row in y..y.saturating_add(h).min(height as u32) {
        for col in x..x.saturating_add(w).min(stride as u32) {
            let idx = row as usize * stride + col as usize;
            if let Some(p) = pixels.get_mut(idx) {
                *p = 0xff;
            }
        }
    }
}

fn hollow_rect(
    pixels: &mut [u8],
    stride: usize,
    height: usize,
    inset_x: u32,
    inset_y: u32,
    cell_w: u32,
    cell_h: u32,
) {
    let inner_w = cell_w.saturating_sub(inset_x * 2);
    let inner_h = cell_h.saturating_sub(inset_y * 2);
    if inner_w == 0 || inner_h == 0 {
        return;
    }
    for row in inset_y..inset_y.saturating_add(inner_h).min(height as u32) {
        for col in inset_x..inset_x.saturating_add(inner_w).min(stride as u32) {
            let idx = row as usize * stride + col as usize;
            if let Some(p) = pixels.get_mut(idx) {
                *p = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtlasFormat, Metrics};

    #[test]
    fn block_sprite_fills_region() {
        let mut atlas = Atlas::new(64, AtlasFormat::Grayscale);
        let opts = RenderOptions {
            grid_metrics: Metrics {
                cell_width: 8,
                cell_height: 16,
                ..Metrics::default()
            },
            ..RenderOptions::default()
        };
        let glyph = render_cursor_sprite(&mut atlas, CursorSprite::Block, &opts).unwrap();
        assert_eq!(glyph.width, 8);
        assert_eq!(glyph.height, 16);
    }
}
