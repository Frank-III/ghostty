use core::ffi::c_int;

use crate::key_setopt_bool_write::key_encoder_setopt_bool_write;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_bool(
    option: c_int,
    value: bool,
    out: *mut bool,
) {
    unsafe { key_encoder_setopt_bool_state(option, value, out) }
}

pub(crate) unsafe fn key_encoder_setopt_bool_state(
    option: c_int,
    value: bool,
    out: *mut bool,
) {
    unsafe { key_encoder_setopt_bool_write(option, value, out) }
}
