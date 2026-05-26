use core::ffi::c_int;

use crate::mouse_button::*;
use crate::mouse_suppress::*;

pub(crate) unsafe fn mouse_required_button_code_or_suppress(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
    out_written: *mut usize,
) -> Result<u8, c_int> {
    match mouse_required_button_code(action, button_present, button, mods, tracking_mode, format) {
        Some(button_code) => Ok(button_code),
        None => Err(unsafe { mouse_suppress_result(out_written) }),
    }
}
