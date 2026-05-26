use core::ffi::c_int;
use crate::mouse_last_cell::*;
use crate::mouse_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_from_terminal(
    event: c_int,
    format: c_int,
    out_event: *mut c_int,
    out_format: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_from_terminal_state(
            event,
            format,
            out_event,
            out_format,
            last_cell_present,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_reset(last_cell_present: *mut bool) {
    unsafe {
        mouse_encoder_reset_state(last_cell_present);
    }
}

pub(crate) unsafe fn mouse_encoder_from_terminal_state(
    event: c_int,
    format: c_int,
    out_event: *mut c_int,
    out_format: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_write_ptr_if_present(out_event, event);
        mouse_write_ptr_if_present(out_format, format);
        mouse_clear_last_cell_present(last_cell_present);
    }
}

pub(crate) unsafe fn mouse_encoder_reset_state(last_cell_present: *mut bool) {
    unsafe {
        mouse_clear_last_cell_present(last_cell_present);
    }
}
