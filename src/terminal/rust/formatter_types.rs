use core::ffi::c_void;

use crate::color_palette::Palette;
use crate::style::GhosttyColorRgb;

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

#[derive(Clone, Copy)]
pub struct Options {
    pub format: Format,
    pub unwrap: bool,
    pub trim: bool,
    pub background: Option<GhosttyColorRgb>,
    pub foreground: Option<GhosttyColorRgb>,
    pub palette: Option<*const Palette>,
}

impl Options {
    pub const PLAIN: Options = Options {
        format: Format::Plain,
        unwrap: false,
        trim: true,
        background: None,
        foreground: None,
        palette: None,
    };

    pub const VT: Options = Options {
        format: Format::Vt,
        unwrap: false,
        trim: true,
        background: None,
        foreground: None,
        palette: None,
    };

    pub const HTML: Options = Options {
        format: Format::Html,
        unwrap: false,
        trim: true,
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
