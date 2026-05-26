use core::ffi::c_int;

use crate::early::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_new(cols: u16, rows: u16) -> c_int {
    terminal_new_impl(cols, rows)
}

pub(crate) fn terminal_new_impl(cols: u16, rows: u16) -> c_int {
    if cols == 0 || rows == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_reset(has_terminal: bool) -> bool {
    terminal_reset_impl(has_terminal)
}

pub(crate) fn terminal_reset_impl(has_terminal: bool) -> bool {
    has_terminal
}
