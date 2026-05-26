use core::ffi::c_int;
use core::ptr;

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
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_reset(last_cell_present: *mut bool) {
    if last_cell_present.is_null() {
        return;
    }

    unsafe {
        ptr::write(last_cell_present, false);
    }
}
