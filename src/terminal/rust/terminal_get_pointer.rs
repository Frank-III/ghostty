use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_pointer(
    data: c_int,
    has_value: bool,
    value: *mut c_void,
    out: *mut c_void,
) -> c_int {
    unsafe { terminal_get_pointer_impl(data, has_value, value, out) }
}

pub(crate) unsafe fn terminal_get_pointer_impl(
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
