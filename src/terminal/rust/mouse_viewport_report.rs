use core::ffi::c_int;

use crate::constants::*;
use crate::input::*;
use crate::mouse_report::*;
use crate::mouse_suppress::*;
use crate::mouse_types::*;
use crate::mouse_viewport::*;

pub(crate) fn mouse_should_suppress_out_of_viewport(
    action: c_int,
    tracking_mode: c_int,
    any_button_pressed: bool,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
) -> bool {
    action != MOUSE_ACTION_RELEASE
        && mouse_pos_out_of_viewport(pos, size)
        && (!mouse_event_sends_motion(tracking_mode) || !any_button_pressed)
}

pub(crate) unsafe fn mouse_viewport_or_suppress(
    action: c_int,
    tracking_mode: c_int,
    any_button_pressed: bool,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    out_written: *mut usize,
) -> Result<(), c_int> {
    if mouse_should_suppress_out_of_viewport(
        action,
        tracking_mode,
        any_button_pressed,
        pos,
        size,
    ) {
        Err(unsafe { mouse_suppress_result(out_written) })
    } else {
        Ok(())
    }
}

pub(crate) unsafe fn mouse_report_viewport_or_suppress(
    action: c_int,
    button_present: bool,
    button: c_int,
    tracking_mode: c_int,
    any_button_pressed: bool,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    out_written: *mut usize,
) -> Result<(), c_int> {
    unsafe {
        mouse_report_or_suppress(action, button_present, button, tracking_mode, out_written)?;
        mouse_viewport_or_suppress(
            action,
            tracking_mode,
            any_button_pressed,
            pos,
            size,
            out_written,
        )
    }
}
