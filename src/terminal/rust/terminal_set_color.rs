use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::palette_copy::*;
use crate::style::*;
use crate::style_write::*;
use crate::terminal::*;

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
