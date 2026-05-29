use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::render::*;
use crate::selection::*;
use crate::style_color_none::*;
use crate::style_default::*;
use crate::style_is_default::*;
use crate::terminal::*;
use core::ffi::c_int;
use core::{mem, ptr};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyleColor {
    pub(crate) tag: c_int,
    pub(crate) value: GhosttyStyleColorValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GhosttyStyleColorValue {
    pub(crate) palette: u8,
    pub(crate) rgb: GhosttyColorRgb,
    pub(crate) padding: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyColorRgb {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyle {
    pub(crate) size: usize,
    pub(crate) fg_color: GhosttyStyleColor,
    pub(crate) bg_color: GhosttyStyleColor,
    pub(crate) underline_color: GhosttyStyleColor,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) faint: bool,
    pub(crate) blink: bool,
    pub(crate) inverse: bool,
    pub(crate) invisible: bool,
    pub(crate) strikethrough: bool,
    pub(crate) overline: bool,
    pub(crate) underline: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_default(result: *mut GhosttyStyle) {
    unsafe { style_default_impl(result) }
}

pub(crate) unsafe fn style_default_impl(result: *mut GhosttyStyle) {
    unsafe {
        write_style_default(result);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_is_default(style: *const GhosttyStyle) -> bool {
    unsafe { style_is_default_impl(style) }
}

pub(crate) unsafe fn style_is_default_impl(style: *const GhosttyStyle) -> bool {
    unsafe { style_is_default(style) }
}
