use core::ffi::c_int;
use core::ptr;

use crate::key_option_as_alt_value::key_option_as_alt_value;
use crate::key_options::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_option_as_alt(
    option: c_int,
    value: c_int,
    out: *mut c_int,
) {
    if option != KEY_ENCODER_OPT_MACOS_OPTION_AS_ALT || out.is_null() {
        return;
    }

    if key_option_as_alt_value(value) {
        unsafe {
            ptr::write(out, value);
        }
    }
}
