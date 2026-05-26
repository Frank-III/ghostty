use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::event_key_field::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_action(event: *mut c_void, action: c_int) {
    unsafe {
        ptr::write(
            key_event_field::<c_int>(event, KEY_EVENT_ACTION_OFFSET),
            action,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_action(event: *mut c_void) -> c_int {
    unsafe { ptr::read(key_event_field::<c_int>(event, KEY_EVENT_ACTION_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_key(event: *mut c_void, key: c_int) {
    unsafe {
        ptr::write(key_event_field::<c_int>(event, KEY_EVENT_KEY_OFFSET), key);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_key(event: *mut c_void) -> c_int {
    unsafe { ptr::read(key_event_field::<c_int>(event, KEY_EVENT_KEY_OFFSET)) }
}
