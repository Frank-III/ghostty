use core::ffi::c_int;

use crate::constants::*;

pub(crate) use crate::mouse_button_mods::*;
pub(crate) use crate::mouse_report::*;

pub(crate) fn mouse_button_code(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
) -> Option<u8> {
    let mut acc = if !button_present {
        3u8
    } else if action == MOUSE_ACTION_RELEASE
        && format != MOUSE_FORMAT_SGR
        && format != MOUSE_FORMAT_SGR_PIXELS
    {
        3u8
    } else {
        match button {
            MOUSE_BUTTON_LEFT => 0,
            MOUSE_BUTTON_MIDDLE => 1,
            MOUSE_BUTTON_RIGHT => 2,
            MOUSE_BUTTON_FOUR => 64,
            MOUSE_BUTTON_FIVE => 65,
            MOUSE_BUTTON_SIX => 66,
            MOUSE_BUTTON_SEVEN => 67,
            MOUSE_BUTTON_EIGHT => 128,
            MOUSE_BUTTON_NINE => 129,
            _ => return None,
        }
    };

    if tracking_mode != MOUSE_TRACKING_X10 {
        acc = mouse_button_apply_mods(acc, mods);
    }

    if action == MOUSE_ACTION_MOTION {
        acc = acc.wrapping_add(32);
    }

    Some(acc)
}
