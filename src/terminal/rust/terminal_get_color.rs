use core::ffi::{c_int, c_void};

use crate::constants::*;
use crate::early::*;
use crate::palette_copy::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_color(
    data: c_int,
    has_value: bool,
    r: u8,
    g: u8,
    b: u8,
    out: *mut c_void,
) -> c_int {
    unsafe { terminal_get_color_impl(data, has_value, r, g, b, out) }
}

pub(crate) unsafe fn terminal_get_color_impl(
    data: c_int,
    has_value: bool,
    r: u8,
    g: u8,
    b: u8,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        write_rgb_value(out.cast::<GhosttyColorRgb>(), r, g, b);
    }
    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_palette(
    data: c_int,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    unsafe { terminal_get_palette_impl(data, palette, out) }
}

pub(crate) unsafe fn terminal_get_palette_impl(
    data: c_int,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_PALETTE | TERMINAL_DATA_COLOR_PALETTE_DEFAULT => unsafe {
            copy_palette(out.cast::<GhosttyColorRgb>(), palette)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}
