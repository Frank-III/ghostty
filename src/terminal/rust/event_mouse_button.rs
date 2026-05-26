use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::event_mouse_field::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_button(event: *mut c_void, button: c_int) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET),
            button,
        );
        ptr::write(
            mouse_event_field::<u32>(event, MOUSE_EVENT_BUTTON_TAG_OFFSET),
            1,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_clear_button(event: *mut c_void) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET),
            0,
        );
        ptr::write(
            mouse_event_field::<u32>(event, MOUSE_EVENT_BUTTON_TAG_OFFSET),
            0,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_button(
    event: *mut c_void,
    out: *mut c_int,
) -> bool {
    let tag = unsafe {
        ptr::read(mouse_event_field::<u32>(
            event,
            MOUSE_EVENT_BUTTON_TAG_OFFSET,
        ))
    };
    if tag == 0 {
        return false;
    }

    if !out.is_null() {
        unsafe {
            ptr::write(
                out,
                ptr::read(mouse_event_field::<c_int>(
                    event,
                    MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET,
                )),
            );
        }
    }

    true
}
