use core::ffi::c_int;

use crate::key_bool_option::key_bool_option;
use crate::key_from_terminal_write::*;

pub(crate) unsafe fn key_encoder_setopt_bool_write(option: c_int, value: bool, out: *mut bool) {
    if key_bool_option(option) {
        unsafe {
            key_write_ptr_if_present(out, value);
        }
    }
}
