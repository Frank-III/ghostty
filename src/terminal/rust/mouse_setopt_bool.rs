use core::ffi::c_int;

use crate::mouse_setopt_bool_write::mouse_encoder_setopt_bool_write;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_bool(
    option: c_int,
    value: bool,
    any_button_pressed: *mut bool,
    track_last_cell: *mut bool,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_bool_write(
            option,
            value,
            any_button_pressed,
            track_last_cell,
            last_cell_present,
        );
    }
}
