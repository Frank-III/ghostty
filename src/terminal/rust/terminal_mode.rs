use core::ffi::c_int;
use core::ptr;

use crate::early::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_mode_get(
    has_terminal: bool,
    has_mode: bool,
    value: bool,
    out: *mut bool,
) -> c_int {
    unsafe { terminal_mode_get_impl(has_terminal, has_mode, value, out) }
}

pub(crate) unsafe fn terminal_mode_get_impl(
    has_terminal: bool,
    has_mode: bool,
    value: bool,
    out: *mut bool,
) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_mode_set(has_terminal: bool, has_mode: bool) -> c_int {
    terminal_mode_set_impl(has_terminal, has_mode)
}

pub(crate) fn terminal_mode_set_impl(has_terminal: bool, has_mode: bool) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}
