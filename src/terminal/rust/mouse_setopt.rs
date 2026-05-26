use core::ffi::c_int;
use core::ptr;

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
        if value != current && !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
        ptr::write(out, value);
    }
}
