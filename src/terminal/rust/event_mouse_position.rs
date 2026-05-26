use core::ffi::c_void;
use core::ptr;

use crate::constants::*;
use crate::event_mouse_field::*;
use crate::mouse_types::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_position(
    event: *mut c_void,
    pos: GhosttyMousePosition,
) {
    unsafe {
        ptr::write(
            mouse_event_field::<GhosttyMousePosition>(event, MOUSE_EVENT_POS_OFFSET),
            pos,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_position(
    event: *mut c_void,
) -> GhosttyMousePosition {
    unsafe {
        ptr::read(mouse_event_field::<GhosttyMousePosition>(
            event,
            MOUSE_EVENT_POS_OFFSET,
        ))
    }
}
