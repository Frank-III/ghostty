use core::ffi::c_int;

use crate::key_option_as_alt_write::key_option_as_alt_write;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_option_as_alt(
    option: c_int,
    value: c_int,
    out: *mut c_int,
) {
    unsafe { key_option_as_alt_state(option, value, out) }
}

pub(crate) unsafe fn key_option_as_alt_state(option: c_int, value: c_int, out: *mut c_int) {
    unsafe { key_option_as_alt_write(option, value, out) }
}
