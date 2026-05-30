//! Codepoint → atlas glyph cache (`src/font/SharedGrid.zig` subset).

use std::collections::HashMap;

use crate::glyph::{Glyph, GlyphRenderError};
use crate::session::FontSession;
use crate::{Atlas, RenderOptions};

/// Caches rasterized glyphs for reuse across draw passes.
#[derive(Debug, Default)]
pub struct GlyphCache {
    glyphs: HashMap<u32, Glyph>,
}

impl GlyphCache {
    pub fn get(&self, cp: u32) -> Option<Glyph> {
        self.glyphs.get(&cp).copied()
    }

    pub fn ensure(
        &mut self,
        session: &FontSession,
        atlas: &mut Atlas,
        cp: u32,
        opts: &RenderOptions,
    ) -> Result<Glyph, GlyphRenderError> {
        if let Some(glyph) = self.glyphs.get(&cp) {
            return Ok(*glyph);
        }
        let glyph = session.render_glyph(atlas, cp, opts)?;
        self.glyphs.insert(cp, glyph);
        Ok(glyph)
    }

    pub fn insert(&mut self, cp: u32, glyph: Glyph) {
        self.glyphs.insert(cp, glyph);
    }

    pub fn warm_snapshot(
        &mut self,
        session: &FontSession,
        atlas: &mut Atlas,
        codepoints: impl IntoIterator<Item = u32>,
        opts: &RenderOptions,
    ) -> Result<(), GlyphRenderError> {
        for cp in codepoints {
            self.ensure(session, atlas, cp, opts)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }
}
