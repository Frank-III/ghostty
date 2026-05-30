//! Build GPU text instances from atlas glyphs (`renderer/cell.zig` subset).

use ghostty_font::GlyphCache;

use crate::cell::{CellAtlas, CellBgDraw, CellText, CellTextBools};
use crate::cells::CellSnapshot;
use crate::color::{shader_rgba, Rgb};

/// Build foreground draw instances for populated cells.
pub fn build_cell_texts(
    snapshot: &CellSnapshot,
    cache: &GlyphCache,
    cell_width: u32,
    cell_height: u32,
) -> Vec<CellText> {
    let mut out = Vec::new();
    let cols = snapshot.grid.columns;
    let rows = snapshot.grid.rows;
    for y in 0..rows {
        for x in 0..cols {
            let Some(cp) = snapshot.codepoints[usize::from(y) * usize::from(cols) + usize::from(x)]
            else {
                continue;
            };
            let idx = usize::from(y) * usize::from(cols) + usize::from(x);
            if snapshot.skip_text_at(idx) {
                continue;
            }
            let Some(glyph) = cache.get(cp) else {
                continue;
            };
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }
            let fg = snapshot
                .foregrounds
                .get(idx)
                .and_then(|c| *c)
                .unwrap_or(Rgb::new(0xff, 0xff, 0xff));
            out.push(CellText {
                glyph_pos: [glyph.atlas_x, glyph.atlas_y],
                glyph_size: [glyph.width, glyph.height],
                bearings: [glyph.offset_x as i16, glyph.offset_y as i16],
                grid_pos: [x, y],
                color: shader_rgba(fg, 0xff),
                atlas: CellAtlas::Grayscale,
                bools: CellTextBools::default(),
            });
        }
    }
    let _ = (cell_width, cell_height);
    out
}

/// Build background draw instances for populated cells.
pub fn build_cell_backgrounds(
    snapshot: &CellSnapshot,
    default_bg: Rgb,
    cell_width: u32,
    cell_height: u32,
) -> Vec<CellBgDraw> {
    let mut out = Vec::new();
    let cols = snapshot.grid.columns;
    let rows = snapshot.grid.rows;
    for y in 0..rows {
        for x in 0..cols {
            let idx = usize::from(y) * usize::from(cols) + usize::from(x);
            let Some(_cp) = snapshot.codepoints.get(idx).and_then(|cp| *cp) else {
                continue;
            };
            let bg = snapshot
                .backgrounds
                .get(idx)
                .and_then(|c| *c)
                .unwrap_or(default_bg);
            out.push(CellBgDraw {
                grid_pos: [x, y],
                color: shader_rgba(bg, 0xff),
            });
        }
    }
    let _ = (cell_width, cell_height);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::GridSize;
    use ghostty_font::{Atlas, AtlasFormat, FontSession, RenderOptions};

    #[test]
    fn build_instances_for_cached_glyph() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, b'X' as u32);

        let descriptor = ghostty_font::descriptor_from_font_family(Some("Menlo"), 12.0);
        let discovered = ghostty_font::select_primary(&descriptor).expect("discover");
        let session =
            FontSession::open(&discovered, ghostty_font::DesiredSize::new(12.0)).expect("open");
        let mut atlas = Atlas::new(128, AtlasFormat::Grayscale);
        let opts = RenderOptions {
            grid_metrics: session.grid_metrics(),
            ..RenderOptions::default()
        };
        let mut cache = GlyphCache::default();
        cache
            .ensure(&session, &mut atlas, b'X' as u32, &opts)
            .expect("glyph");

        let texts = build_cell_texts(&snap, &cache, 8, 16);
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].grid_pos, [0, 0]);
        assert!(texts[0].glyph_size[0] > 0);
        assert_eq!(texts[0].color[0..3], [0xff, 0xff, 0xff]);
    }

    #[test]
    fn build_instances_use_cell_foreground() {
        use crate::color::Rgb;
        let grid = GridSize {
            columns: 1,
            rows: 1,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, b'X' as u32);
        snap.set_foreground(0, 0, Rgb::new(0x11, 0x22, 0x33));

        let descriptor = ghostty_font::descriptor_from_font_family(Some("Menlo"), 12.0);
        let discovered = ghostty_font::select_primary(&descriptor).expect("discover");
        let session =
            FontSession::open(&discovered, ghostty_font::DesiredSize::new(12.0)).expect("open");
        let mut atlas = Atlas::new(128, AtlasFormat::Grayscale);
        let opts = RenderOptions {
            grid_metrics: session.grid_metrics(),
            ..RenderOptions::default()
        };
        let mut cache = GlyphCache::default();
        cache
            .ensure(&session, &mut atlas, b'X' as u32, &opts)
            .expect("glyph");

        let texts = build_cell_texts(&snap, &cache, 8, 16);
        assert_eq!(texts[0].color[0..3], [0x11, 0x22, 0x33]);
    }

    #[test]
    fn build_instances_skip_spacer_tail() {
        let grid = GridSize {
            columns: 2,
            rows: 1,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, 0x4e16);
        snap.set_wide(0, 0, 1);
        snap.set(1, 0, 0);
        snap.set_wide(1, 0, 2);

        let cache = GlyphCache::default();
        let texts = build_cell_texts(&snap, &cache, 8, 16);
        assert_eq!(texts.len(), 0);
    }

    #[test]
    fn build_background_instances_use_cell_color() {
        use crate::color::Rgb;
        let grid = GridSize {
            columns: 1,
            rows: 1,
        };
        let mut snap = CellSnapshot::empty(grid);
        snap.set(0, 0, b'X' as u32);
        snap.set_background(0, 0, Rgb::new(0x40, 0x41, 0x42));

        let bgs = build_cell_backgrounds(&snap, Rgb::new(0, 0, 0), 8, 16);
        assert_eq!(bgs.len(), 1);
        assert_eq!(bgs[0].grid_pos, [0, 0]);
        assert_eq!(bgs[0].color[0..3], [0x40, 0x41, 0x42]);
    }
}
