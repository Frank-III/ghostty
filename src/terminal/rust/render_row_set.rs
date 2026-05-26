use core::ffi::c_int;
use core::ptr;

use crate::constants::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_set_dirty(value: bool, out: *mut bool) -> c_int {
    unsafe { render_row_set_dirty_impl(value, out) }
}

pub(crate) unsafe fn render_row_set_dirty_impl(value: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_set(
    has_iterator: bool,
    has_row: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    render_row_set_impl(has_iterator, has_row, option, has_value)
}

pub(crate) fn render_row_set_impl(
    has_iterator: bool,
    has_row: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    if !has_iterator || !has_row || !has_value {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match option {
        RENDER_ROW_SET_DIRTY => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}
