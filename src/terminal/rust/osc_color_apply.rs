#![allow(unused)]

use crate::color_palette::DynamicPalette;
use crate::osc_parser_color::{ColorRequest, ColorTarget, DynamicColor};
use crate::simple_write::{write_bytes, write_decimal};
use crate::stream_types::{ColorOscOp, OscTerminator};
use crate::style::GhosttyColorRgb;
use crate::style_types::RGB;
use crate::terminal_types::TerminalColors;

const REPORT_NONE: u8 = 0;
const REPORT_16BIT: u8 = 1;
const REPORT_8BIT: u8 = 2;

pub(crate) fn rgb_to_ghostty(rgb: RGB) -> GhosttyColorRgb {
    GhosttyColorRgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

pub(crate) fn ghostty_to_rgb(color: GhosttyColorRgb) -> RGB {
    RGB {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

fn resolve_color(colors: &TerminalColors, target: ColorTarget) -> Option<RGB> {
    match target {
        ColorTarget::Palette(i) => {
            let c = colors.palette.current()[i as usize];
            Some(ghostty_to_rgb(c))
        }
        ColorTarget::Dynamic(dynamic) => dynamic_color(colors, dynamic).map(ghostty_to_rgb),
        ColorTarget::Special(_) => None,
    }
}

fn dynamic_color(colors: &TerminalColors, dynamic: DynamicColor) -> Option<GhosttyColorRgb> {
    match dynamic {
        DynamicColor::Foreground => colors.foreground.get(),
        DynamicColor::Background => colors.background.get(),
        DynamicColor::Cursor => colors.cursor.get(),
        _ => None,
    }
}

fn apply_set(colors: &mut TerminalColors, ct: crate::osc_parser_color::ColoredTarget) {
    let color = rgb_to_ghostty(ct.color);
    match ct.target {
        ColorTarget::Palette(i) => {
            colors.palette.set(i, color);
        }
        ColorTarget::Dynamic(DynamicColor::Foreground) => {
            colors.foreground.set_override(Some(color))
        }
        ColorTarget::Dynamic(DynamicColor::Background) => {
            colors.background.set_override(Some(color))
        }
        ColorTarget::Dynamic(DynamicColor::Cursor) => colors.cursor.set_override(Some(color)),
        _ => {}
    }
}

fn apply_reset(colors: &mut TerminalColors, target: ColorTarget) {
    match target {
        ColorTarget::Palette(i) => colors.palette.reset(i),
        ColorTarget::Dynamic(DynamicColor::Foreground) => colors.foreground.set_override(None),
        ColorTarget::Dynamic(DynamicColor::Background) => colors.background.set_override(None),
        ColorTarget::Dynamic(DynamicColor::Cursor) => colors.cursor.set_override(None),
        _ => {}
    }
}

fn write_hex4(out: &mut [u8], off: &mut usize, value: u8, report_format: u8) {
    let expanded = u16::from(value) * 257;
    let mut nibbles = [0u8; 4];
    nibbles[0] = hex_char((expanded >> 12) & 0xf);
    nibbles[1] = hex_char((expanded >> 8) & 0xf);
    nibbles[2] = hex_char((expanded >> 4) & 0xf);
    nibbles[3] = hex_char(expanded & 0xf);
    if report_format == REPORT_8BIT {
        unsafe {
            write_bytes(out.as_mut_ptr(), off, &[value]);
        }
    } else {
        unsafe {
            write_bytes(out.as_mut_ptr(), off, &nibbles);
        }
    }
}

fn hex_char(n: u16) -> u8 {
    match n {
        0..=9 => b'0' + n as u8,
        10..=15 => b'a' + (n - 10) as u8,
        _ => b'0',
    }
}

fn write_terminator(out: &mut [u8], off: &mut usize, terminator: OscTerminator) {
    match terminator {
        OscTerminator::St => unsafe {
            write_bytes(out.as_mut_ptr(), off, b"\x1b\\");
        },
        OscTerminator::Bel => unsafe {
            write_bytes(out.as_mut_ptr(), off, b"\x07");
        },
    }
}

fn encode_palette_query(
    out: &mut [u8],
    off: &mut usize,
    index: u8,
    color: RGB,
    report_format: u8,
    terminator: OscTerminator,
) -> bool {
    unsafe {
        write_bytes(out.as_mut_ptr(), off, b"\x1b]4;");
        write_decimal(out.as_mut_ptr(), off, index as u64);
        write_bytes(out.as_mut_ptr(), off, b";rgb:");
        write_hex4(out, off, color.r, report_format);
        write_bytes(out.as_mut_ptr(), off, b"/");
        write_hex4(out, off, color.g, report_format);
        write_bytes(out.as_mut_ptr(), off, b"/");
        write_hex4(out, off, color.b, report_format);
    }
    write_terminator(out, off, terminator);
    true
}

fn dynamic_osc_id(dynamic: DynamicColor) -> Option<u8> {
    Some(dynamic as u8)
}

fn encode_dynamic_query(
    out: &mut [u8],
    off: &mut usize,
    dynamic: DynamicColor,
    color: RGB,
    report_format: u8,
    terminator: OscTerminator,
) -> bool {
    let Some(id) = dynamic_osc_id(dynamic) else {
        return false;
    };
    unsafe {
        write_bytes(out.as_mut_ptr(), off, b"\x1b]");
        write_decimal(out.as_mut_ptr(), off, id as u64);
        write_bytes(out.as_mut_ptr(), off, b";rgb:");
        write_hex4(out, off, color.r, report_format);
        write_bytes(out.as_mut_ptr(), off, b"/");
        write_hex4(out, off, color.g, report_format);
        write_bytes(out.as_mut_ptr(), off, b"/");
        write_hex4(out, off, color.b, report_format);
    }
    write_terminator(out, off, terminator);
    true
}

pub(crate) fn handle_color_requests(
    colors: &mut TerminalColors,
    requests: &[ColorRequest],
    report_format: u8,
    terminator: OscTerminator,
    mut write: impl FnMut(&[u8]),
    mut color_change: impl FnMut(ColorTarget, RGB),
) {
    if requests.is_empty() {
        return;
    }

    let mut buf = [0u8; 1024];
    let mut off = 0usize;

    for req in requests {
        match *req {
            ColorRequest::Set(ct) => {
                apply_set(colors, ct);
                color_change(ct.target, ct.color);
            }
            ColorRequest::Reset(target) => {
                apply_reset(colors, target);
                if let Some(color) = resolve_color(colors, target) {
                    color_change(target, color);
                }
            }
            ColorRequest::ResetPalette => colors.palette.reset_all(),
            ColorRequest::ResetSpecial => {}
            ColorRequest::Query(target) => {
                if report_format == REPORT_NONE {
                    continue;
                }
                let Some(color) = resolve_color(colors, target) else {
                    continue;
                };
                let wrote = match target {
                    ColorTarget::Palette(i) => encode_palette_query(
                        &mut buf,
                        &mut off,
                        i,
                        color,
                        report_format,
                        terminator,
                    ),
                    ColorTarget::Dynamic(dynamic) => encode_dynamic_query(
                        &mut buf,
                        &mut off,
                        dynamic,
                        color,
                        report_format,
                        terminator,
                    ),
                    _ => false,
                };
                if wrote && off > 0 {
                    write(&buf[..off]);
                    off = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc_parser_color::parse_color_osc;
    use crate::stream_types::{ColorOscOp, OscTerminator};

    #[test]
    fn osc4_query_encodes_response() {
        let mut colors = TerminalColors::default_val();
        colors
            .palette
            .set(0, rgb_to_ghostty(RGB::new(0x01, 0x02, 0x03)));
        let parsed = parse_color_osc(ColorOscOp::Osc4, b"0;?", OscTerminator::St);
        let mut out = Vec::new();
        handle_color_requests(
            &mut colors,
            parsed.requests.as_slice(),
            REPORT_16BIT,
            OscTerminator::St,
            |chunk| out.extend_from_slice(chunk),
            |_target, _color| {},
        );
        let text = std::str::from_utf8(&out).unwrap();
        assert!(text.starts_with("\x1b]4;0;rgb:"));
        assert!(text.ends_with("\x1b\\"));
    }
}
