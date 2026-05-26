use core::ffi::c_int;
use core::ptr;

use crate::mouse_last_cell::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_from_terminal(
    event: c_int,
    format: c_int,
    out_event: *mut c_int,
    out_format: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_event.is_null() {
            ptr::write(out_event, event);
        }
        if !out_format.is_null() {
            ptr::write(out_format, format);
        }
        mouse_clear_last_cell_present(last_cell_present);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_reset(last_cell_present: *mut bool) {
    unsafe {
        mouse_clear_last_cell_present(last_cell_present);
    }
}
