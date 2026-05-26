use core::ffi::c_int;

use crate::constants::*;
use crate::mouse_last_cell::*;
use crate::mouse_write::*;

pub(crate) unsafe fn mouse_encoder_setopt_bool_write(
    option: c_int,
    value: bool,
    any_button_pressed: *mut bool,
    track_last_cell: *mut bool,
    last_cell_present: *mut bool,
) {
    unsafe {
        match option {
            MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED => {
                mouse_write_ptr_if_present(any_button_pressed, value);
            }
            MOUSE_ENCODER_OPT_TRACK_LAST_CELL => {
                mouse_write_ptr_if_present(track_last_cell, value);
                if !value {
                    mouse_clear_last_cell_present(last_cell_present);
                }
            }
            _ => {}
        }
    }
}
