use core::ffi::c_int;

use crate::constants::*;
use crate::input::*;
use crate::mouse_report::*;
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
