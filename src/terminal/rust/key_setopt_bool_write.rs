use core::ffi::c_int;
use core::ptr;

use crate::key_bool_option::key_bool_option;

pub(crate) unsafe fn key_encoder_setopt_bool_write(option: c_int, value: bool, out: *mut bool) {
    if out.is_null() {
        return;
    }

    if key_bool_option(option) {
        unsafe {
            ptr::write(out, value);
        }
    }
}
