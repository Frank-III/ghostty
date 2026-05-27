use core::ffi::c_void;


use crate::color_palette::Palette;
use crate::style::GhosttyColorRgb;

#[derive(Clone, Copy)]
pub enum CodepointReplacement {
    Codepoint(u32),
    String { data: *const u8, len: usize },
}

#[derive(Clone, Copy)]
pub struct CodepointMapEntry {
    pub range_start: u32,
    pub range_end: u32,
    pub replacement: CodepointReplacement,
}

#[derive(Clone, Copy)]
pub struct CodepointMap {
    pub entries: *const CodepointMapEntry,
    pub len: usize,
}

impl CodepointMap {
    pub const EMPTY: CodepointMap = CodepointMap {
        entries: core::ptr::null(),
        len: 0,
    };

    pub fn find_replacement(&self, codepoint: u32) -> Option<CodepointReplacement> {
        let len = self.len;
        if len == 0 || self.entries.is_null() {
            return None;
        }
        let mut i = len;
        unsafe {
            while i > 0 {
                i -= 1;
                let entry = &*self.entries.add(i);
                if entry.range_start <= codepoint && codepoint <= entry.range_end {
                    return Some(entry.replacement);
                }
            }
        }
        None
    }
}

pub type PinMapAppendFn = fn(ctx: *mut c_void, pin: *const c_void, count: usize) -> bool;

#[derive(Clone, Copy)]
pub struct PinMap {
    pub append_fn: PinMapAppendFn,
    pub ctx: *mut c_void,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Plain = 0,
    Vt = 1,
    Html = 2,
}

pub trait Writer {
    fn write(&mut self, bytes: &[u8]) -> usize;
}

pub fn format_styled(f: Format) -> bool {
    match f {
        Format::Plain => false,
        Format::Vt | Format::Html => true,
    }
}

pub struct Options {
    pub format: Format,
    pub unwrap: bool,
    pub trim: bool,
    pub codepoint_map: CodepointMap,
    pub background: Option<GhosttyColorRgb>,
    pub foreground: Option<GhosttyColorRgb>,
    pub palette: Option<*const Palette>,
}

impl Clone for Options {
    fn clone(&self) -> Self { *self }
}

impl Copy for Options {}

impl Options {
    pub const PLAIN: Options = Options {
        format: Format::Plain,
        unwrap: false,
        trim: true,
        codepoint_map: CodepointMap::EMPTY,
        background: None,
        foreground: None,
        palette: None,
    };

    pub const VT: Options = Options {
        format: Format::Vt,
        unwrap: false,
        trim: true,
        codepoint_map: CodepointMap::EMPTY,
        background: None,
        foreground: None,
        palette: None,
    };

    pub const HTML: Options = Options {
        format: Format::Html,
        unwrap: false,
        trim: true,
        codepoint_map: CodepointMap::EMPTY,
        background: None,
        foreground: None,
        palette: None,
    };
}

#[derive(Clone, Copy)]
pub struct ScreenContent {
    pub is_none: bool,
    pub selection: *const c_void,
}

impl ScreenContent {
    pub const NONE: ScreenContent = ScreenContent {
        is_none: true,
        selection: core::ptr::null(),
    };

    pub const ALL: ScreenContent = ScreenContent {
        is_none: false,
        selection: core::ptr::null(),
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScreenExtra {
    pub cursor: bool,
    pub style: bool,
    pub hyperlink: bool,
    pub protection: bool,
    pub kitty_keyboard: bool,
    pub charsets: bool,
}

impl ScreenExtra {
    pub const NONE: ScreenExtra = ScreenExtra {
        cursor: false,
        style: false,
        hyperlink: false,
        protection: false,
        kitty_keyboard: false,
        charsets: false,
    };

    pub const STYLES: ScreenExtra = ScreenExtra {
        cursor: false,
        style: true,
        hyperlink: true,
        protection: false,
        kitty_keyboard: false,
        charsets: false,
    };

    pub const ALL: ScreenExtra = ScreenExtra {
        cursor: true,
        style: true,
        hyperlink: true,
        protection: true,
        kitty_keyboard: true,
        charsets: true,
    };

    pub fn is_set(self) -> bool {
        self.cursor
            || self.style
            || self.hyperlink
            || self.protection
            || self.kitty_keyboard
            || self.charsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_map_find_replacement_single_codepoint() {
        let entries = [CodepointMapEntry {
            range_start: 0x2603,
            range_end: 0x2603,
            replacement: CodepointReplacement::Codepoint(0x2764),
        }];
        let map = CodepointMap { entries: entries.as_ptr(), len: 1 };
        match map.find_replacement(0x2603) {
            Some(CodepointReplacement::Codepoint(cp)) => assert_eq!(cp, 0x2764),
            other => panic!("expected CodepointReplacement::Codepoint(0x2764), got {:?}", other.is_some()),
        }
        assert!(map.find_replacement(0x41).is_none());
    }

    #[test]
    fn codepoint_map_find_replacement_utf8_string() {
        let repl = " ";
        let entries = [CodepointMapEntry {
            range_start: 0x00A0,
            range_end: 0x00A0,
            replacement: CodepointReplacement::String {
                data: repl.as_ptr(),
                len: repl.len(),
            },
        }];
        let map = CodepointMap { entries: entries.as_ptr(), len: 1 };
        match map.find_replacement(0x00A0) {
            Some(CodepointReplacement::String { data, len }) => {
                let bytes = unsafe { core::slice::from_raw_parts(data, len) };
                assert_eq!(bytes, b" ");
            }
            other => panic!("expected CodepointReplacement::String, got {:?}", other.is_some()),
        }
        assert!(map.find_replacement(0x20).is_none());
    }

    #[test]
    fn codepoint_map_last_match_wins() {
        let entries = [
            CodepointMapEntry {
                range_start: 0x0000,
                range_end: 0xFFFF,
                replacement: CodepointReplacement::Codepoint(0x41),
            },
            CodepointMapEntry {
                range_start: 0x2603,
                range_end: 0x2603,
                replacement: CodepointReplacement::Codepoint(0x42),
            },
        ];
        let map = CodepointMap { entries: entries.as_ptr(), len: 2 };
        match map.find_replacement(0x2603) {
            Some(CodepointReplacement::Codepoint(cp)) => assert_eq!(cp, 0x42),
            other => panic!("expected Codepoint(0x42) (last entry), got {:?}", other.is_some()),
        }
        match map.find_replacement(0x41) {
            Some(CodepointReplacement::Codepoint(cp)) => assert_eq!(cp, 0x41),
            other => panic!("expected Codepoint(0x41) (first entry), got {:?}", other.is_some()),
        }
    }

    #[test]
    fn codepoint_map_empty_returns_none() {
        assert!(CodepointMap::EMPTY.find_replacement(0x41).is_none());
        assert!(CodepointMap::EMPTY.find_replacement(0x2603).is_none());
        assert!(CodepointMap::EMPTY.find_replacement(0x00A0).is_none());
    }
}
