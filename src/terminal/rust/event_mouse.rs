use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::event_mouse_field::*;
use crate::input::*;
use crate::mouse_encode::*;
use crate::mouse_types::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_init(event: *mut c_void) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_ACTION_OFFSET),
            MOUSE_ACTION_PRESS,
        );
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET),
            0,
        );
        ptr::write(
            mouse_event_field::<u32>(event, MOUSE_EVENT_BUTTON_TAG_OFFSET),
            0,
        );
        ptr::write(
            mouse_event_field::<GhosttyMousePosition>(event, MOUSE_EVENT_POS_OFFSET),
            GhosttyMousePosition { x: 0.0, y: 0.0 },
        );
        ptr::write(mouse_event_field::<u16>(event, MOUSE_EVENT_MODS_OFFSET), 0);
    }
}
