use core::ffi::c_int;
use core::ptr;

use crate::key_bool_option::key_bool_option;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_bool(
    option: c_int,
    value: bool,
    out: *mut bool,
) {
    if out.is_null() {
        return;
    }

    if key_bool_option(option) {
        unsafe {
            ptr::write(out, value);
        }
    }
}
