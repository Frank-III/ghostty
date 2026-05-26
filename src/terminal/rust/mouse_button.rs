use core::ffi::c_int;

use crate::constants::*;

pub(crate) use crate::mouse_button_base::*;
pub(crate) use crate::mouse_button_default::*;
pub(crate) use crate::mouse_button_motion::*;
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
    let mut acc = mouse_button_default_code(action, button_present, format)
        .or_else(|| mouse_button_base_code(button))?;

    if tracking_mode != MOUSE_TRACKING_X10 {
        acc = mouse_button_apply_mods(acc, mods);
    }

    acc = mouse_button_apply_motion(acc, action);

    Some(acc)
}

pub(crate) fn mouse_required_button_code(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
) -> Option<u8> {
    mouse_button_code(action, button_present, button, mods, tracking_mode, format)
}
