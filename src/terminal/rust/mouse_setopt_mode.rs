use core::ffi::c_int;

use crate::mouse_setopt::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_event(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_format(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}
