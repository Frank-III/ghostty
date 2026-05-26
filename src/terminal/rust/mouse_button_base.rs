use core::ffi::c_int;

use crate::constants::*;

pub(crate) fn mouse_button_base_code(button: c_int) -> Option<u8> {
    match button {
        MOUSE_BUTTON_LEFT => Some(0),
        MOUSE_BUTTON_MIDDLE => Some(1),
        MOUSE_BUTTON_RIGHT => Some(2),
        MOUSE_BUTTON_FOUR => Some(64),
        MOUSE_BUTTON_FIVE => Some(65),
        MOUSE_BUTTON_SIX => Some(66),
        MOUSE_BUTTON_SEVEN => Some(67),
        MOUSE_BUTTON_EIGHT => Some(128),
        MOUSE_BUTTON_NINE => Some(129),
        _ => None,
    }
}
