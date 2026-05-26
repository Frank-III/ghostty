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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_action(event: *mut c_void, action: c_int) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_ACTION_OFFSET),
            action,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_action(event: *mut c_void) -> c_int {
    unsafe { ptr::read(mouse_event_field::<c_int>(event, MOUSE_EVENT_ACTION_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(
            mouse_event_field::<u16>(event, MOUSE_EVENT_MODS_OFFSET),
            mods,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_mods(event: *mut c_void) -> u16 {
    unsafe { ptr::read(mouse_event_field::<u16>(event, MOUSE_EVENT_MODS_OFFSET)) }
}

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
