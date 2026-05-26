use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;

pub(crate) const GHOSTTY_SUCCESS: c_int = 0;
pub(crate) const GHOSTTY_INVALID_VALUE: c_int = -2;
pub(crate) const GHOSTTY_OUT_OF_SPACE: c_int = -3;
pub(crate) const GHOSTTY_NO_VALUE: c_int = -4;

pub(crate) const GHOSTTY_FOCUS_LOST: c_int = 1;
#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_hyperlink_uri(
    has_uri: bool,
    uri: *const u8,
    uri_len: usize,
    out_buf: *mut u8,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        ptr::write(out_len, if has_uri { uri_len } else { 0 });
    }

    if !has_uri {
        return GHOSTTY_SUCCESS;
    }

    if out_buf.is_null() || buf_len < uri_len {
        return GHOSTTY_OUT_OF_SPACE;
    }

    let mut i = 0usize;
    while i < uri_len {
        unsafe {
            ptr::write(out_buf.add(i), ptr::read(uri.add(i)));
        }
        i += 1;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_graphemes(
    has_text: bool,
    codepoint: u32,
    out_buf: *mut u32,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        ptr::write(out_len, if has_text { 1 } else { 0 });
    }

    if !has_text {
        return GHOSTTY_SUCCESS;
    }

    if out_buf.is_null() || buf_len < 1 {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        ptr::write(out_buf, codepoint);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_has_value(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
) -> bool {
    has_ref && has_page_list && !garbage
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_result(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
    has_point: bool,
) -> c_int {
    if !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    if !has_page_list || garbage || !has_point {
        return GHOSTTY_NO_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_set_input(
    has_ref: bool,
    has_terminal: bool,
    same_terminal: bool,
) -> c_int {
    if !has_ref || !has_terminal || !same_terminal {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}
