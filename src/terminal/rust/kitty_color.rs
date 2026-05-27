#![allow(unused)]

use crate::stream_types::OscTerminator;
use crate::style_types::RGB;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Special {
    Foreground = 0,
    Background = 1,
    SelectionForeground = 2,
    SelectionBackground = 3,
    Cursor = 4,
    CursorText = 5,
    VisualBell = 6,
    SecondTransparentBackground = 7,
}

impl Special {
    #[inline]
    pub fn from_bytes(s: &[u8]) -> Option<Self> {
        match s {
            b"foreground" => Some(Special::Foreground),
            b"background" => Some(Special::Background),
            b"selection_foreground" => Some(Special::SelectionForeground),
            b"selection_background" => Some(Special::SelectionBackground),
            b"cursor" => Some(Special::Cursor),
            b"cursor_text" => Some(Special::CursorText),
            b"visual_bell" => Some(Special::VisualBell),
            b"second_transparent_background" => Some(Special::SecondTransparentBackground),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Special::Foreground => b"foreground",
            Special::Background => b"background",
            Special::SelectionForeground => b"selection_foreground",
            Special::SelectionBackground => b"selection_background",
            Special::Cursor => b"cursor",
            Special::CursorText => b"cursor_text",
            Special::VisualBell => b"visual_bell",
            Special::SecondTransparentBackground => b"second_transparent_background",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Palette(u8),
    Special(Special),
}

impl Kind {
    pub const MAX: usize = 255 + 8;

    #[inline]
    pub fn parse(key: &[u8]) -> Option<Self> {
        if let Some(s) = Special::from_bytes(key) {
            return Some(Kind::Special(s));
        }
        match parse_u8(key) {
            Some(v) => Some(Kind::Palette(v)),
            None => None,
        }
    }

    pub unsafe fn write_to(self, out: *mut u8, offset: &mut usize) {
        match self {
            Kind::Palette(p) => unsafe { write_decimal(out, offset, p as u64) },
            Kind::Special(s) => unsafe { write_bytes(out, offset, s.as_bytes()) },
        }
    }

    pub fn write_len(self) -> usize {
        match self {
            Kind::Palette(p) => decimal_len(p as u64),
            Kind::Special(s) => s.as_bytes().len(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Query(Kind),
    Set { key: Kind, color: RGB },
    Reset(Kind),
}

pub struct Osc<'a> {
    pub requests: &'a [Request],
    pub terminator: OscTerminator,
}

impl<'a> Default for Osc<'a> {
    fn default() -> Self {
        Self {
            requests: &[],
            terminator: OscTerminator::St,
        }
    }
}

fn parse_u8(s: &[u8]) -> Option<u8> {
    if s.is_empty() || s.len() > 3 {
        return None;
    }
    let mut result: u16 = 0;
    let mut i = 0;
    while i < s.len() {
        let byte = unsafe { *s.get_unchecked(i) };
        if byte < b'0' || byte > b'9' {
            return None;
        }
        result = result * 10 + (byte - b'0') as u16;
        i += 1;
    }
    if result > 255 {
        return None;
    }
    Some(result as u8)
}

fn decimal_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

unsafe fn write_decimal(out: *mut u8, offset: &mut usize, mut value: u64) {
    let mut reversed = [0u8; 20];
    let mut len = 0usize;

    loop {
        let digit = (value % 10) as u8;
        unsafe {
            core::ptr::write(reversed.as_mut_ptr().add(len), b'0' + digit);
        }
        len += 1;
        value /= 10;
        if value == 0 {
            break;
        }
    }

    while len > 0 {
        len -= 1;
        let byte = unsafe { core::ptr::read(reversed.as_ptr().add(len)) };
        unsafe {
            core::ptr::write(out.add(*offset), byte);
        }
        *offset += 1;
    }
}

unsafe fn write_bytes(out: *mut u8, offset: &mut usize, bytes: &[u8]) {
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = unsafe { core::ptr::read(bytes.as_ptr().add(i)) };
        unsafe {
            core::ptr::write(out.add(*offset + i), byte);
        }
        i += 1;
    }
    *offset += bytes.len();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_special_parse() {
        let k = Kind::parse(b"foreground");
        assert!(k == Some(Kind::Special(Special::Foreground)));
    }

    #[test]
    fn kind_palette_parse() {
        let k = Kind::parse(b"42");
        assert!(k == Some(Kind::Palette(42)));
    }

    #[test]
    fn kind_parse_invalid() {
        assert!(Kind::parse(b"").is_none());
        assert!(Kind::parse(b"abc").is_none());
        assert!(Kind::parse(b"256").is_none());
    }

    #[test]
    fn kind_write_special() {
        let mut buf = [0u8; 64];
        let mut offset = 0usize;
        unsafe { Kind::Special(Special::Foreground).write_to(buf.as_mut_ptr(), &mut offset) };
        assert!(offset == 10);
        assert!(&buf[..10] == b"foreground");
    }

    #[test]
    fn kind_write_palette() {
        let mut buf = [0u8; 64];
        let mut offset = 0usize;
        unsafe { Kind::Palette(42).write_to(buf.as_mut_ptr(), &mut offset) };
        assert!(offset == 2);
        assert!(&buf[..2] == b"42");
    }
}
