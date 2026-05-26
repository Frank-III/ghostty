use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::event_key_field::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(key_event_field::<u16>(event, KEY_EVENT_MODS_OFFSET), mods);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_mods(event: *mut c_void) -> u16 {
    unsafe { ptr::read(key_event_field::<u16>(event, KEY_EVENT_MODS_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_consumed_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(
            key_event_field::<u16>(event, KEY_EVENT_CONSUMED_MODS_OFFSET),
            mods,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_consumed_mods(event: *mut c_void) -> u16 {
    unsafe {
        ptr::read(key_event_field::<u16>(
            event,
            KEY_EVENT_CONSUMED_MODS_OFFSET,
        ))
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_composing(event: *mut c_void, composing: bool) {
    unsafe {
        ptr::write(
            key_event_field::<bool>(event, KEY_EVENT_COMPOSING_OFFSET),
            composing,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_composing(event: *mut c_void) -> bool {
    unsafe { ptr::read(key_event_field::<bool>(event, KEY_EVENT_COMPOSING_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_init(event: *mut c_void) {
    unsafe {
        ptr::write(key_event_field::<*const u8>(event, KEY_EVENT_UTF8_PTR_OFFSET), b"".as_ptr());
        ptr::write(key_event_field::<usize>(event, KEY_EVENT_UTF8_LEN_OFFSET), 0);
        ptr::write(key_event_field::<c_int>(event, KEY_EVENT_ACTION_OFFSET), 1);
        ptr::write(key_event_field::<c_int>(event, KEY_EVENT_KEY_OFFSET), 0);
        ptr::write(key_event_field::<u32>(event, KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET), 0);
        ptr::write(key_event_field::<u16>(event, KEY_EVENT_MODS_OFFSET), 0);
        ptr::write(key_event_field::<u16>(event, KEY_EVENT_CONSUMED_MODS_OFFSET), 0);
        ptr::write(key_event_field::<bool>(event, KEY_EVENT_COMPOSING_OFFSET), false);
    }
}
