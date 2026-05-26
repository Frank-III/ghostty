use core::ffi::c_int;

use crate::mouse_last_cell::*;
use crate::mouse_write::*;

pub(crate) unsafe fn mouse_encoder_setopt_mode(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    if out.is_null() {
        return;
    }

    unsafe {
        if value != current {
            mouse_clear_last_cell_present(last_cell_present);
        }
        mouse_write_ptr(out, value);
    }
}
