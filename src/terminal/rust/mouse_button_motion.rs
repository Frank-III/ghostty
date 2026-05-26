use core::ffi::c_int;

use crate::constants::*;

pub(crate) fn mouse_button_apply_motion(mut code: u8, action: c_int) -> u8 {
    if action == MOUSE_ACTION_MOTION {
        code = code.wrapping_add(32);
    }

    code
}
