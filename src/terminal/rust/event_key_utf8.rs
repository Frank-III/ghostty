use core::ffi::c_void;
use core::ptr;

use crate::constants::*;
use crate::event_key_field::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_utf8(
    event: *mut c_void,
    utf8: *const u8,
    len: usize,
) {
    unsafe {
        key_event_set_utf8_impl(event, utf8, len);
    }
}

pub(crate) unsafe fn key_event_set_utf8_impl(event: *mut c_void, utf8: *const u8, len: usize) {
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
    unsafe { key_event_get_utf8_impl(event, len) }
}

pub(crate) unsafe fn key_event_get_utf8_impl(event: *mut c_void, len: *mut usize) -> *const u8 {
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
