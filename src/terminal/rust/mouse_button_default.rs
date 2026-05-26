use core::ffi::c_int;

use crate::constants::*;

pub(crate) fn mouse_button_default_code(
    action: c_int,
    button_present: bool,
    format: c_int,
) -> Option<u8> {
    if !button_present {
        Some(3)
    } else if action == MOUSE_ACTION_RELEASE
        && format != MOUSE_FORMAT_SGR
        && format != MOUSE_FORMAT_SGR_PIXELS
    {
        Some(3)
    } else {
        None
    }
}
