use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::selection_copy::*;
use crate::simple::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_string(
    data: c_int,
    ptr: *const u8,
    len: usize,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_TITLE | TERMINAL_DATA_PWD => unsafe { write_borrowed_string(out, ptr, len) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_style(
    data: c_int,
    style: *const GhosttyStyle,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_CURSOR_STYLE => unsafe { copy_style(out.cast::<GhosttyStyle>(), style) },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scrollbar(
    data: c_int,
    total: u64,
    offset: u64,
    len: u64,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SCROLLBAR => unsafe {
            write_scrollbar(out.cast::<GhosttyTerminalScrollbar>(), total, offset, len)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_pointer(
    data: c_int,
    has_value: bool,
    value: *mut c_void,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_KITTY_GRAPHICS => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        ptr::write(out.cast::<*mut c_void>(), value);
    }

    GHOSTTY_SUCCESS
}
