use core::ffi::c_int;
use core::ptr;

use crate::constants::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_next(
    has_current: bool,
    current: u16,
    len: usize,
    out_next: *mut u16,
) -> bool {
    unsafe { render_index_next_impl(has_current, current, len, out_next) }
}

pub(crate) unsafe fn render_index_next_impl(
    has_current: bool,
    current: u16,
    len: usize,
    out_next: *mut u16,
) -> bool {
    if out_next.is_null() {
        return false;
    }

    let next = if has_current {
        match current.checked_add(1) {
            Some(value) => value,
            None => return false,
        }
    } else {
        0
    };

    if usize::from(next) >= len {
        return false;
    }

    unsafe {
        ptr::write(out_next, next);
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_select(index: u16, len: usize) -> c_int {
    render_index_select_impl(index, len)
}

pub(crate) fn render_index_select_impl(index: u16, len: usize) -> c_int {
    if usize::from(index) >= len {
        return RENDER_RESULT_INVALID_VALUE;
    }

    RENDER_RESULT_SUCCESS
}
