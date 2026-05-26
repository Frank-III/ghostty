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
pub unsafe extern "C" fn ghostty_rust_key_event_set_utf8(
    event: *mut c_void,
    utf8: *const u8,
    len: usize,
) {
    let ptr = if utf8.is_null() {
        EMPTY_UTF8.as_ptr()
    } else {
        utf8
    };
    unsafe {
        ptr::write(
            key_event_field::<*const u8>(event, KEY_EVENT_UTF8_PTR_OFFSET),
            ptr,
        );
        ptr::write(
            key_event_field::<usize>(event, KEY_EVENT_UTF8_LEN_OFFSET),
            len,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_utf8(
    event: *mut c_void,
    len: *mut usize,
) -> *const u8 {
    let utf8_len = unsafe { ptr::read(key_event_field::<usize>(event, KEY_EVENT_UTF8_LEN_OFFSET)) };
    if !len.is_null() {
        unsafe {
            ptr::write(len, utf8_len);
        }
    }

    if utf8_len == 0 {
        ptr::null()
    } else {
        unsafe {
            ptr::read(key_event_field::<*const u8>(
                event,
                KEY_EVENT_UTF8_PTR_OFFSET,
            ))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_unshifted_codepoint(
    event: *mut c_void,
    codepoint: u32,
) {
    unsafe {
        ptr::write(
            key_event_field::<u32>(event, KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET),
            codepoint & 0x001f_ffff,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_unshifted_codepoint(event: *mut c_void) -> u32 {
    unsafe {
        ptr::read(key_event_field::<u32>(
            event,
            KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET,
        ))
    }
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
