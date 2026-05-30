//! Build cursor draw instances and uniforms for the GPU path.

use ghostty_font::{Atlas, CursorSprite, FontSession, GlyphCache, RenderOptions};

use crate::cell::{CellAtlas, CellText, CellTextBools};
use crate::cells::CellSnapshot;
use crate::color::{shader_rgba, Rgb};
use crate::cursor::{
    resolve_style, CursorStyle, CursorStyleOptions, RenderCursorState, TerminalCursorStyle,
};
use crate::uniforms::CursorUniforms;

/// Inputs for cursor draw resolution (from terminal + surface state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorDrawInput {
    pub viewport_visible: bool,
    pub viewport_x: u16,
    pub viewport_y: u16,
    pub viewport_wide_tail: bool,
    pub visible: bool,
    pub blinking: bool,
    pub password_input: bool,
    pub visual_style: u8,
    pub cursor_rgb: Option<[u8; 3]>,
    pub focused: bool,
    pub blink_visible: bool,
    pub preedit: bool,
}

impl CursorDrawInput {
    pub fn render_state(&self) -> RenderCursorState {
        RenderCursorState {
            viewport_visible: self.viewport_visible,
            visible: self.visible,
            blinking: self.blinking,
            password_input: self.password_input,
            visual_style: terminal_style_from_raw(self.visual_style),
        }
    }
}

fn terminal_style_from_raw(raw: u8) -> TerminalCursorStyle {
    match raw {
        0 => TerminalCursorStyle::Bar,
        2 => TerminalCursorStyle::Underline,
        3 => TerminalCursorStyle::BlockHollow,
        _ => TerminalCursorStyle::Block,
    }
}

/// Result of preparing cursor geometry for a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorDraw {
    pub style: CursorStyle,
    pub grid_pos: [u16; 2],
    pub cell: CellText,
    pub uniforms: Option<CursorUniforms>,
}

/// Resolve cursor style and rasterize the cursor sprite when visible.
pub fn build_cursor_draw(
    input: CursorDrawInput,
    snapshot: &CellSnapshot,
    session: &FontSession,
    cache: &mut GlyphCache,
    atlas: &mut Atlas,
    opts: &RenderOptions,
    default_fg: Rgb,
    default_bg: Rgb,
) -> Option<CursorDraw> {
    let _ = session;
    let style = resolve_style(
        &input.render_state(),
        CursorStyleOptions {
            preedit: input.preedit,
            focused: input.focused,
            blink_visible: input.blink_visible,
        },
    )?;

    let mut x = input.viewport_x;
    let y = input.viewport_y;
    if input.viewport_wide_tail && x > 0 {
        x -= 1;
    }

    let cols = snapshot.grid.columns;
    let idx = usize::from(y) * usize::from(cols) + usize::from(x);
    let cell_fg = snapshot
        .foregrounds
        .get(idx)
        .and_then(|c| *c)
        .unwrap_or(default_fg);
    let cell_bg = snapshot
        .backgrounds
        .get(idx)
        .and_then(|c| *c)
        .unwrap_or(default_bg);

    let cursor_color = input
        .cursor_rgb
        .map(|rgb| Rgb::new(rgb[0], rgb[1], rgb[2]))
        .unwrap_or(match style {
            CursorStyle::Block => cell_fg,
            _ => cell_bg,
        });

    let sprite = match style {
        CursorStyle::Lock => CursorSprite::Block,
        other => CursorSprite::from_visual_style(match other {
            CursorStyle::Bar => 0,
            CursorStyle::Underline => 2,
            CursorStyle::BlockHollow => 3,
            _ => 1,
        }),
    };

    let cache_key = cursor_cache_key(style);
    let glyph = if let Some(g) = cache.get(cache_key) {
        g
    } else {
        let g = ghostty_font::render_cursor_sprite(atlas, sprite, opts).ok()?;
        cache.insert(cache_key, g);
        g
    };

    let wide = snapshot.grid_columns_at(idx) > 1;
    let cell = CellText {
        glyph_pos: [glyph.atlas_x, glyph.atlas_y],
        glyph_size: [glyph.width, glyph.height],
        bearings: [glyph.offset_x as i16, glyph.offset_y as i16],
        grid_pos: [x, y],
        color: shader_rgba(cursor_color, 0xff),
        atlas: CellAtlas::Grayscale,
        bools: CellTextBools::new(false, true),
    };

    let uniforms = if style == CursorStyle::Block {
        Some(CursorUniforms {
            grid_pos: [x, y],
            color: shader_rgba(cursor_color, 0xff),
            wide,
        })
    } else {
        None
    };

    Some(CursorDraw {
        style,
        grid_pos: [x, y],
        cell,
        uniforms,
    })
}

fn cursor_cache_key(style: CursorStyle) -> u32 {
    match style {
        CursorStyle::Block => 0xE000_0001,
        CursorStyle::BlockHollow => 0xE000_0002,
        CursorStyle::Bar => 0xE000_0003,
        CursorStyle::Underline => 0xE000_0004,
        CursorStyle::Lock => 0xE000_0005,
    }
}
