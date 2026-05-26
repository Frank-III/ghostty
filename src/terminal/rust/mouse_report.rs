use core::ffi::c_int;

use crate::constants::*;
use crate::mouse_suppress::*;

pub(crate) fn mouse_should_report(
    action: c_int,
    button_present: bool,
    button: c_int,
    tracking_mode: c_int,
) -> bool {
    match tracking_mode {
        MOUSE_TRACKING_NONE => false,
        MOUSE_TRACKING_X10 => {
            action == MOUSE_ACTION_PRESS
                && button_present
                && (button == MOUSE_BUTTON_LEFT
                    || button == MOUSE_BUTTON_MIDDLE
                    || button == MOUSE_BUTTON_RIGHT)
        }
        MOUSE_TRACKING_NORMAL => action != MOUSE_ACTION_MOTION,
        MOUSE_TRACKING_BUTTON => button_present,
        MOUSE_TRACKING_ANY => true,
        _ => false,
    }
}

pub(crate) fn mouse_event_sends_motion(tracking_mode: c_int) -> bool {
    tracking_mode == MOUSE_TRACKING_BUTTON || tracking_mode == MOUSE_TRACKING_ANY
}

pub(crate) unsafe fn mouse_report_or_suppress(
    action: c_int,
    button_present: bool,
    button: c_int,
    tracking_mode: c_int,
    out_written: *mut usize,
) -> Result<(), c_int> {
    if mouse_should_report(action, button_present, button, tracking_mode) {
        Ok(())
    } else {
        Err(unsafe { mouse_suppress_result(out_written) })
    }
}
