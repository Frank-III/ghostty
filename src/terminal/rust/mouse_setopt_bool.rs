use core::ffi::c_int;
use core::ptr;

use crate::constants::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_bool(
    option: c_int,
    value: bool,
    any_button_pressed: *mut bool,
    track_last_cell: *mut bool,
    last_cell_present: *mut bool,
) {
    unsafe {
        match option {
            MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED => {
                if !any_button_pressed.is_null() {
                    ptr::write(any_button_pressed, value);
                }
            }
            MOUSE_ENCODER_OPT_TRACK_LAST_CELL => {
                if !track_last_cell.is_null() {
                    ptr::write(track_last_cell, value);
                }
                if !value && !last_cell_present.is_null() {
                    ptr::write(last_cell_present, false);
                }
            }
            _ => {}
        }
    }
}
