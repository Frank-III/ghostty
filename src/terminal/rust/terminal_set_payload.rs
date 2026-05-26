use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::simple::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_string(
    value: *const GhosttyString,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> c_int {
    if out_ptr.is_null() || out_len.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (ptr, len) = if value.is_null() {
        (EMPTY_UTF8.as_ptr(), 0)
    } else {
        unsafe {
            (
                ptr::read(core::ptr::addr_of!((*value).ptr)),
                ptr::read(core::ptr::addr_of!((*value).len)),
            )
        }
    };

    unsafe {
        ptr::write(out_ptr, ptr);
        ptr::write(out_len, len);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_rgb(
    value: *const GhosttyColorRgb,
    out_has_value: *mut bool,
    out_rgb: *mut GhosttyColorRgb,
) -> c_int {
    if out_has_value.is_null() || out_rgb.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let rgb = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        write_rgb_value(out_rgb, rgb.r, rgb.g, rgb.b);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_palette(
    value: *const GhosttyColorRgb,
    out_has_value: *mut bool,
    out_palette: *mut *const GhosttyColorRgb,
) -> c_int {
    if out_has_value.is_null() || out_palette.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_palette, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_u64_zero(
    value: *const u64,
    out_value: *mut u64,
) -> c_int {
    if out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let decoded = if value.is_null() {
        0
    } else {
        unsafe { ptr::read(value) }
    };

    unsafe {
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_bool_optional(
    value: *const bool,
    out_has_value: *mut bool,
    out_value: *mut bool,
) -> c_int {
    if out_has_value.is_null() || out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let decoded = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_usize_optional(
    value: *const usize,
    out_has_value: *mut bool,
    out_value: *mut usize,
) -> c_int {
    if out_has_value.is_null() || out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let decoded = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}
