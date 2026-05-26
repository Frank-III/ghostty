use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::event_mouse_field::*;

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
