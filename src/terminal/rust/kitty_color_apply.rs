#![allow(unused)]

use crate::kitty_color::{Kind, Request, Special};
use crate::osc_color_apply::rgb_to_ghostty;
use crate::style_types::RGB;
use crate::terminal_types::{DynamicRgb, TerminalColors, TerminalDirty, TerminalFlags};

pub(crate) fn apply_kitty_color_requests(
    colors: &mut TerminalColors,
    dirty: &mut TerminalDirty,
    requests: &[Request],
    mut color_change: impl FnMut(Kind, RGB),
) {
    for req in requests {
        match *req {
            Request::Set { key, color } => {
                apply_set(colors, dirty, key, color);
                color_change(key, color);
            }
            Request::Reset(key) => {
                apply_reset(colors, dirty, key);
                if let Some(rgb) = resolve_kitty_color(colors, key) {
                    color_change(key, rgb);
                }
            }
            Request::Query(_) => {}
        }
    }
}

fn resolve_kitty_color(colors: &TerminalColors, key: Kind) -> Option<RGB> {
    match key {
        Kind::Palette(i) => {
            let c = colors.palette.current()[i as usize];
            Some(crate::osc_color_apply::ghostty_to_rgb(c))
        }
        Kind::Special(Special::Foreground) => colors.foreground.get().map(crate::osc_color_apply::ghostty_to_rgb),
        Kind::Special(Special::Background) => colors.background.get().map(crate::osc_color_apply::ghostty_to_rgb),
        Kind::Special(Special::Cursor) => colors.cursor.get().map(crate::osc_color_apply::ghostty_to_rgb),
        _ => None,
    }
}

fn apply_set(colors: &mut TerminalColors, dirty: &mut TerminalDirty, key: Kind, color: RGB) {
    let ghostty = rgb_to_ghostty(color);
    match key {
        Kind::Palette(index) => {
            dirty.palette = true;
            colors.palette.set(index, ghostty);
        }
        Kind::Special(special) => match special {
            Special::Foreground => set_dynamic(&mut colors.foreground, ghostty),
            Special::Background => set_dynamic(&mut colors.background, ghostty),
            Special::Cursor => set_dynamic(&mut colors.cursor, ghostty),
            _ => {}
        },
    }
}

fn apply_reset(colors: &mut TerminalColors, dirty: &mut TerminalDirty, key: Kind) {
    match key {
        Kind::Palette(index) => {
            dirty.palette = true;
            colors.palette.reset(index);
        }
        Kind::Special(special) => match special {
            Special::Foreground => reset_dynamic(&mut colors.foreground),
            Special::Background => reset_dynamic(&mut colors.background),
            Special::Cursor => reset_dynamic(&mut colors.cursor),
            _ => {}
        },
    }
}

fn set_dynamic(slot: &mut DynamicRgb, color: crate::style::GhosttyColorRgb) {
    slot.set_override(Some(color));
}

fn reset_dynamic(slot: &mut DynamicRgb) {
    if let Some(def) = slot.default_color() {
        slot.set_override(Some(def));
    } else {
        slot.set_override(None);
    }
}

pub(crate) fn apply_kitty_color_requests_with_flags(
    colors: &mut TerminalColors,
    flags: &mut TerminalFlags,
    requests: &[Request],
    color_change: impl FnMut(Kind, RGB),
) {
    apply_kitty_color_requests(colors, &mut flags.dirty, requests, color_change);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc_parser_kitty::parse_kitty_color;
    use crate::stream_types::OscTerminator;

    #[test]
    fn kitty_set_background() {
        let mut colors = TerminalColors::default_val();
        let mut dirty = TerminalDirty::default();
        let parsed = parse_kitty_color(b"background=#aabbcc", OscTerminator::St);
        apply_kitty_color_requests(&mut colors, &mut dirty, parsed.as_slice(), |_k, _c| {});
        let bg = colors.background.get().expect("background");
        assert!(bg.r == 0xaa && bg.g == 0xbb && bg.b == 0xcc);
    }

    #[test]
    fn kitty_reset_foreground() {
        let mut colors = TerminalColors::default_val();
        colors
            .foreground
            .set_default(Some(rgb_to_ghostty(RGB::new(1, 2, 3))));
        colors
            .foreground
            .set_override(Some(rgb_to_ghostty(RGB::new(4, 5, 6))));
        let mut dirty = TerminalDirty::default();
        let parsed = parse_kitty_color(b"foreground", OscTerminator::St);
        apply_kitty_color_requests(&mut colors, &mut dirty, parsed.as_slice(), |_k, _c| {});
        let fg = colors.foreground.get().expect("foreground");
        assert!(fg.r == 1 && fg.g == 2 && fg.b == 3);
    }

    #[test]
    fn kitty_palette_set_marks_dirty() {
        let mut colors = TerminalColors::default_val();
        let mut dirty = TerminalDirty::default();
        let parsed = parse_kitty_color(b"2=#ff0000", OscTerminator::St);
        apply_kitty_color_requests(&mut colors, &mut dirty, parsed.as_slice(), |_k, _c| {});
        assert!(dirty.palette);
        let c = colors.palette.current()[2];
        assert!(c.r == 255 && c.g == 0 && c.b == 0);
    }
}
