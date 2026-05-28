#![allow(unused)]

use crate::stream_types::{ColorOscOp, OscTerminator};
use crate::style_types::RGB;
use crate::x11_color;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpecialColor {
    Foreground = 0,
    Background = 1,
    Cursor = 2,
    PointerForeground = 3,
    PointerBackground = 4,
    TektronixForeground = 5,
    TektronixBackground = 6,
    HighlightBackground = 7,
}

impl SpecialColor {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        if v <= 7 {
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DynamicColor {
    Foreground = 10,
    Background = 11,
    Cursor = 12,
    PointerForeground = 13,
    PointerBackground = 14,
    TektronixForeground = 15,
    TektronixBackground = 16,
    HighlightBackground = 17,
    TektronixCursor = 18,
    HighlightForeground = 19,
}

impl DynamicColor {
    #[inline]
    pub fn from_osc_num(n: u8) -> Option<Self> {
        if n >= 10 && n <= 19 {
            Some(unsafe { core::mem::transmute(n) })
        } else {
            None
        }
    }

    #[inline]
    pub fn next(self) -> Option<Self> {
        let v = self as u8 + 1;
        if v <= 19 {
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorTarget {
    Palette(u8),
    Special(SpecialColor),
    Dynamic(DynamicColor),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColoredTarget {
    pub target: ColorTarget,
    pub color: RGB,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorRequest {
    Set(ColoredTarget),
    Query(ColorTarget),
    Reset(ColorTarget),
    ResetPalette,
    ResetSpecial,
}

pub const COLOR_MAX_REQUESTS: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColorRequestList {
    pub items: [ColorRequest; COLOR_MAX_REQUESTS],
    pub len: usize,
}

impl ColorRequestList {
    #[inline]
    pub fn new() -> Self {
        Self {
            items: [ColorRequest::ResetPalette; COLOR_MAX_REQUESTS],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, req: ColorRequest) -> bool {
        if self.len >= COLOR_MAX_REQUESTS {
            return false;
        }
        unsafe {
            *self.items.get_unchecked_mut(self.len) = req;
        }
        self.len += 1;
        true
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_slice(&self) -> &[ColorRequest] {
        unsafe { self.items.get_unchecked(..self.len) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColorOscResult {
    pub op: ColorOscOp,
    pub requests: ColorRequestList,
    pub terminator: OscTerminator,
}

impl ColorOscResult {
    pub fn new(op: ColorOscOp, terminator: OscTerminator) -> Self {
        Self {
            op,
            requests: ColorRequestList::new(),
            terminator,
        }
    }
}

fn hex_digit(b: u8) -> Option<u16> {
    match b {
        b'0'..=b'9' => Some((b - b'0') as u16),
        b'a'..=b'f' => Some((b - b'a' + 10) as u16),
        b'A'..=b'F' => Some((b - b'A' + 10) as u16),
        _ => None,
    }
}

fn parse_hex_value(s: &[u8]) -> Option<u16> {
    if s.is_empty() || s.len() > 4 {
        return None;
    }
    let mut result: u16 = 0;
    let mut i = 0;
    while i < s.len() {
        let d = hex_digit(unsafe { *s.get_unchecked(i) })?;
        result = result * 16 + d;
        i += 1;
    }
    Some(result)
}

fn from_hex(s: &[u8]) -> Option<u8> {
    if s.is_empty() || s.len() > 4 {
        return None;
    }
    let color = parse_hex_value(s)?;
    let divisor: u32 = match s.len() {
        1 => 15,
        2 => 255,
        3 => 4095,
        4 => 65535,
        _ => return None,
    };
    Some(((color as u32) * 255 / divisor) as u8)
}

fn parse_f64_simple(s: &[u8]) -> Option<f64> {
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    let negative = unsafe { *s.get_unchecked(0) } == b'-';
    if negative {
        i = 1;
    }
    let mut int_part: u64 = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b == b'.' {
            break;
        }
        if b < b'0' || b > b'9' {
            return None;
        }
        int_part = int_part * 10 + (b - b'0') as u64;
        i += 1;
    }
    let mut result = int_part as f64;
    if negative {
        result = -result;
    }
    if i < s.len() && unsafe { *s.get_unchecked(i) } == b'.' {
        i += 1;
        let mut frac: f64 = 0.0;
        let mut divisor: f64 = 1.0;
        while i < s.len() {
            let b = unsafe { *s.get_unchecked(i) };
            if b < b'0' || b > b'9' {
                return None;
            }
            frac = frac * 10.0 + (b - b'0') as f64;
            divisor *= 10.0;
            i += 1;
        }
        if negative {
            result -= frac / divisor;
        } else {
            result += frac / divisor;
        }
    }
    Some(result)
}

fn from_intensity(s: &[u8]) -> Option<u8> {
    let v = parse_f64_simple(s)?;
    if v < 0.0 || v > 1.0 {
        return None;
    }
    Some((v * 255.0) as u8)
}

fn find_byte(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < haystack.len() {
        if unsafe { *haystack.get_unchecked(i) } == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn trim_whitespace(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end {
        let b = unsafe { *s.get_unchecked(start) };
        if b != b' ' && b != b'\t' {
            break;
        }
        start += 1;
    }
    while end > start {
        let b = unsafe { *s.get_unchecked(end - 1) };
        if b != b' ' && b != b'\t' {
            break;
        }
        end -= 1;
    }
    unsafe { s.get_unchecked(start..end) }
}

fn bytes_eq_icase(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        let ca = unsafe { *a.get_unchecked(i) };
        let cb = unsafe { *b.get_unchecked(i) };
        let la = if ca >= b'A' && ca <= b'Z' { ca + 32 } else { ca };
        let lb = if cb >= b'A' && cb <= b'Z' { cb + 32 } else { cb };
        if la != lb {
            return false;
        }
        i += 1;
    }
    true
}

pub fn rgb_parse(value: &[u8]) -> Option<RGB> {
    if value.is_empty() {
        return None;
    }

    let trimmed = trim_whitespace(value);
    if trimmed.is_empty() {
        return None;
    }

    if unsafe { *trimmed.get_unchecked(0) } == b'#' {
        return match trimmed.len() {
            4 => {
                let r = from_hex(unsafe { trimmed.get_unchecked(1..2) })?;
                let g = from_hex(unsafe { trimmed.get_unchecked(2..3) })?;
                let b = from_hex(unsafe { trimmed.get_unchecked(3..4) })?;
                Some(RGB::new(r, g, b))
            }
            7 => {
                let r = from_hex(unsafe { trimmed.get_unchecked(1..3) })?;
                let g = from_hex(unsafe { trimmed.get_unchecked(3..5) })?;
                let b = from_hex(unsafe { trimmed.get_unchecked(5..7) })?;
                Some(RGB::new(r, g, b))
            }
            10 => {
                let r = from_hex(unsafe { trimmed.get_unchecked(1..4) })?;
                let g = from_hex(unsafe { trimmed.get_unchecked(4..7) })?;
                let b = from_hex(unsafe { trimmed.get_unchecked(7..10) })?;
                Some(RGB::new(r, g, b))
            }
            13 => {
                let r = from_hex(unsafe { trimmed.get_unchecked(1..5) })?;
                let g = from_hex(unsafe { trimmed.get_unchecked(5..9) })?;
                let b = from_hex(unsafe { trimmed.get_unchecked(9..13) })?;
                Some(RGB::new(r, g, b))
            }
            _ => None,
        };
    }

    if let Some(s) = crate::bytes_util::bytes_to_str_opt(trimmed) {
        if let Some(rgb) = x11_color::get(s) {
            return Some(rgb);
        }
    }

    if trimmed.len() < 9 {
        return None;
    }
    if !bytes_eq_icase(unsafe { trimmed.get_unchecked(0..3) }, b"rgb") {
        return None;
    }

    let mut i: usize = 3;
    let use_intensity = if i < trimmed.len() && unsafe { *trimmed.get_unchecked(i) } == b'i' {
        i += 1;
        true
    } else {
        false
    };

    if i >= trimmed.len() || unsafe { *trimmed.get_unchecked(i) } != b':' {
        return None;
    }
    i += 1;

    let slash1 = find_byte(trimmed, b'/', i)?;
    let r_slice = unsafe { trimmed.get_unchecked(i..slash1) };
    i = slash1 + 1;

    let slash2 = find_byte(trimmed, b'/', i)?;
    let g_slice = unsafe { trimmed.get_unchecked(i..slash2) };
    i = slash2 + 1;

    let b_slice = unsafe { trimmed.get_unchecked(i..) };

    let r = if use_intensity { from_intensity(r_slice)? } else { from_hex(r_slice)? };
    let g = if use_intensity { from_intensity(g_slice)? } else { from_hex(g_slice)? };
    let b = if use_intensity { from_intensity(b_slice)? } else { from_hex(b_slice)? };

    Some(RGB::new(r, g, b))
}

fn parse_u9(s: &[u8]) -> Option<u16> {
    if s.is_empty() || s.len() > 3 {
        return None;
    }
    let mut result: u16 = 0;
    let mut i = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b < b'0' || b > b'9' {
            return None;
        }
        result = result * 10 + (b - b'0') as u16;
        i += 1;
    }
    if result > 511 {
        return None;
    }
    Some(result)
}

fn next_token<'a>(data: &'a [u8], pos: &mut usize) -> Option<&'a [u8]> {
    if *pos > data.len() {
        return None;
    }
    let start = *pos;
    let mut i = start;
    while i < data.len() {
        if unsafe { *data.get_unchecked(i) } == b';' {
            let token = unsafe { data.get_unchecked(start..i) };
            *pos = i + 1;
            return Some(token);
        }
        i += 1;
    }
    if start <= data.len() {
        let token = unsafe { data.get_unchecked(start..) };
        *pos = data.len() + 1;
        return Some(token);
    }
    None
}

fn has_more(data: &[u8], pos: usize) -> bool {
    pos <= data.len()
}

fn make_ansi_target(op: ColorOscOp, color: u16) -> Option<ColorTarget> {
    match op {
        ColorOscOp::Osc5 => {
            let idx = if color <= 7 { color as u8 } else { return None };
            Some(ColorTarget::Special(SpecialColor::from_u8(idx)?))
        }
        ColorOscOp::Osc4 => {
            if color <= 255 {
                Some(ColorTarget::Palette(color as u8))
            } else {
                let offset = color - 256;
                let idx = if offset <= 7 { offset as u8 } else { return None };
                Some(ColorTarget::Special(SpecialColor::from_u8(idx)?))
            }
        }
        _ => None,
    }
}

fn make_reset_ansi_target(op: ColorOscOp, color: u16) -> Option<ColorTarget> {
    make_ansi_target(op, color)
}

fn parse_get_set_ansi_color(op: ColorOscOp, data: &[u8], list: &mut ColorRequestList) {
    let mut pos: usize = 0;
    loop {
        let color_str = match next_token(data, &mut pos) {
            Some(t) => t,
            None => return,
        };
        let spec_str = match next_token(data, &mut pos) {
            Some(t) => t,
            None => return,
        };

        let color = match parse_u9(color_str) {
            Some(c) => c,
            None => return,
        };

        let target = match make_ansi_target(op, color) {
            Some(t) => t,
            None => return,
        };

        if spec_str.len() == 1 && unsafe { *spec_str.get_unchecked(0) } == b'?' {
            if !list.push(ColorRequest::Query(target)) {
                return;
            }
            continue;
        }

        let rgb = match rgb_parse(spec_str) {
            Some(c) => c,
            None => return,
        };
        if !list.push(ColorRequest::Set(ColoredTarget { target, color: rgb })) {
            return;
        }
    }
}

fn parse_reset_ansi_color(op: ColorOscOp, data: &[u8], list: &mut ColorRequestList) {
    let mut pos: usize = 0;
    let mut any = false;

    loop {
        let color_str = match next_token(data, &mut pos) {
            Some(t) => t,
            None => break,
        };

        if color_str.is_empty() {
            if !has_more(data, pos) {
                break;
            }
            continue;
        }

        let color = match parse_u9(color_str) {
            Some(c) => c,
            None => continue,
        };

        let target = match make_reset_ansi_target(op, color) {
            Some(t) => t,
            None => continue,
        };

        if !list.push(ColorRequest::Reset(target)) {
            return;
        }
        any = true;
    }

    if !any {
        let req = match op {
            ColorOscOp::Osc104 => ColorRequest::ResetPalette,
            ColorOscOp::Osc105 => ColorRequest::ResetSpecial,
            _ => return,
        };
        let _ = list.push(req);
    }
}

fn parse_get_set_dynamic_color(start: DynamicColor, data: &[u8], list: &mut ColorRequestList) {
    let mut pos: usize = 0;
    let mut color = start;

    loop {
        let spec_str = match next_token(data, &mut pos) {
            Some(t) => t,
            None => return,
        };

        let target = ColorTarget::Dynamic(color);

        if spec_str.len() == 1 && unsafe { *spec_str.get_unchecked(0) } == b'?' {
            if !list.push(ColorRequest::Query(target)) {
                return;
            }
        } else {
            let rgb = match rgb_parse(spec_str) {
                Some(c) => c,
                None => return,
            };
            if !list.push(ColorRequest::Set(ColoredTarget { target, color: rgb })) {
                return;
            }
        }

        match color.next() {
            Some(c) => color = c,
            None => return,
        }
    }
}

fn parse_reset_dynamic_color(color: DynamicColor, data: &[u8], list: &mut ColorRequestList) {
    let mut pos: usize = 0;
    if next_token(data, &mut pos).is_some() && has_more(data, pos) {
        return;
    }
    let _ = list.push(ColorRequest::Reset(ColorTarget::Dynamic(color)));
}

pub fn parse_color_osc(op: ColorOscOp, data: &[u8], terminator: OscTerminator) -> ColorOscResult {
    let mut result = ColorOscResult::new(op, terminator);

    match op {
        ColorOscOp::Osc4 | ColorOscOp::Osc5 => {
            parse_get_set_ansi_color(op, data, &mut result.requests);
        }
        ColorOscOp::Osc104 | ColorOscOp::Osc105 => {
            parse_reset_ansi_color(op, data, &mut result.requests);
        }
        ColorOscOp::Osc10 => parse_get_set_dynamic_color(DynamicColor::Foreground, data, &mut result.requests),
        ColorOscOp::Osc11 => parse_get_set_dynamic_color(DynamicColor::Background, data, &mut result.requests),
        ColorOscOp::Osc12 => parse_get_set_dynamic_color(DynamicColor::Cursor, data, &mut result.requests),
        ColorOscOp::Osc13 => parse_get_set_dynamic_color(DynamicColor::PointerForeground, data, &mut result.requests),
        ColorOscOp::Osc14 => parse_get_set_dynamic_color(DynamicColor::PointerBackground, data, &mut result.requests),
        ColorOscOp::Osc15 => parse_get_set_dynamic_color(DynamicColor::TektronixForeground, data, &mut result.requests),
        ColorOscOp::Osc16 => parse_get_set_dynamic_color(DynamicColor::TektronixBackground, data, &mut result.requests),
        ColorOscOp::Osc17 => parse_get_set_dynamic_color(DynamicColor::HighlightBackground, data, &mut result.requests),
        ColorOscOp::Osc18 => parse_get_set_dynamic_color(DynamicColor::TektronixCursor, data, &mut result.requests),
        ColorOscOp::Osc19 => parse_get_set_dynamic_color(DynamicColor::HighlightForeground, data, &mut result.requests),
        ColorOscOp::Osc110 => parse_reset_dynamic_color(DynamicColor::Foreground, data, &mut result.requests),
        ColorOscOp::Osc111 => parse_reset_dynamic_color(DynamicColor::Background, data, &mut result.requests),
        ColorOscOp::Osc112 => parse_reset_dynamic_color(DynamicColor::Cursor, data, &mut result.requests),
        ColorOscOp::Osc113 => parse_reset_dynamic_color(DynamicColor::PointerForeground, data, &mut result.requests),
        ColorOscOp::Osc114 => parse_reset_dynamic_color(DynamicColor::PointerBackground, data, &mut result.requests),
        ColorOscOp::Osc115 => parse_reset_dynamic_color(DynamicColor::TektronixForeground, data, &mut result.requests),
        ColorOscOp::Osc116 => parse_reset_dynamic_color(DynamicColor::TektronixBackground, data, &mut result.requests),
        ColorOscOp::Osc117 => parse_reset_dynamic_color(DynamicColor::HighlightBackground, data, &mut result.requests),
        ColorOscOp::Osc118 => parse_reset_dynamic_color(DynamicColor::TektronixCursor, data, &mut result.requests),
        ColorOscOp::Osc119 => parse_reset_dynamic_color(DynamicColor::HighlightForeground, data, &mut result.requests),
        _ => {}
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_parse_hex_short() {
        let c = rgb_parse(b"#f00").unwrap();
        assert!(c.r == 255 && c.g == 0 && c.b == 0);
    }

    #[test]
    fn rgb_parse_hex_full() {
        let c = rgb_parse(b"#ff0000").unwrap();
        assert!(c.r == 255 && c.g == 0 && c.b == 0);
    }

    #[test]
    fn rgb_parse_rgb_prefix() {
        let c = rgb_parse(b"rgb:ff/00/00").unwrap();
        assert!(c.r == 255 && c.g == 0 && c.b == 0);
    }

    #[test]
    fn rgb_parse_rgbi() {
        let c = rgb_parse(b"rgbi:1.0/1.0/1.0").unwrap();
        assert!(c.r == 255 && c.g == 255 && c.b == 255);
    }

    #[test]
    fn rgb_parse_invalid() {
        assert!(rgb_parse(b"").is_none());
        assert!(rgb_parse(b"#xyz").is_none());
    }

    #[test]
    fn osc4_set_palette() {
        let result = parse_color_osc(ColorOscOp::Osc4, b"0;red", OscTerminator::St);
        assert!(result.requests.count() == 1);
        match result.requests.as_slice()[0] {
            ColorRequest::Set(ct) => {
                assert!(ct.target == ColorTarget::Palette(0));
                assert!(ct.color.r == 255 && ct.color.g == 0 && ct.color.b == 0);
            }
            _ => panic!("expected Set"),
        }
    }

    #[test]
    fn osc4_query() {
        let result = parse_color_osc(ColorOscOp::Osc4, b"0;?", OscTerminator::St);
        assert!(result.requests.count() == 1);
        match result.requests.as_slice()[0] {
            ColorRequest::Query(t) => assert!(t == ColorTarget::Palette(0)),
            _ => panic!("expected Query"),
        }
    }

    #[test]
    fn osc4_multiple() {
        let result = parse_color_osc(ColorOscOp::Osc4, b"0;red;1;blue", OscTerminator::St);
        assert!(result.requests.count() == 2);
    }

    #[test]
    fn osc104_reset_all() {
        let result = parse_color_osc(ColorOscOp::Osc104, b"", OscTerminator::St);
        assert!(result.requests.count() == 1);
        assert!(result.requests.as_slice()[0] == ColorRequest::ResetPalette);
    }

    #[test]
    fn osc104_reset_specific() {
        let result = parse_color_osc(ColorOscOp::Osc104, b"0", OscTerminator::St);
        assert!(result.requests.count() == 1);
        match result.requests.as_slice()[0] {
            ColorRequest::Reset(t) => assert!(t == ColorTarget::Palette(0)),
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn osc10_set_dynamic() {
        let result = parse_color_osc(ColorOscOp::Osc10, b"red", OscTerminator::St);
        assert!(result.requests.count() == 1);
        match result.requests.as_slice()[0] {
            ColorRequest::Set(ct) => {
                assert!(ct.target == ColorTarget::Dynamic(DynamicColor::Foreground));
            }
            _ => panic!("expected Set"),
        }
    }

    #[test]
    fn osc110_reset_dynamic() {
        let result = parse_color_osc(ColorOscOp::Osc110, b"", OscTerminator::St);
        assert!(result.requests.count() == 1);
        match result.requests.as_slice()[0] {
            ColorRequest::Reset(t) => assert!(t == ColorTarget::Dynamic(DynamicColor::Foreground)),
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn dynamic_color_next() {
        let d = DynamicColor::Foreground;
        let n = d.next().unwrap();
        assert!(n == DynamicColor::Background);
    }

    #[test]
    fn special_color_from_u8() {
        assert!(SpecialColor::from_u8(0) == Some(SpecialColor::Foreground));
        assert!(SpecialColor::from_u8(7) == Some(SpecialColor::HighlightBackground));
        assert!(SpecialColor::from_u8(8).is_none());
    }
}
