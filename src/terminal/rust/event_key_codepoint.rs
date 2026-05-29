use core::ffi::c_void;
use core::ptr;

use crate::constants::*;
use crate::event_key_field::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_unshifted_codepoint(
    event: *mut c_void,
    codepoint: u32,
) {
    unsafe {
        key_event_set_unshifted_codepoint_impl(event, codepoint);
    }
}

pub(crate) unsafe fn key_event_set_unshifted_codepoint_impl(event: *mut c_void, codepoint: u32) {
    unsafe {
        ptr::write(
            key_event_field::<u32>(event, KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET),
            codepoint & 0x001f_ffff,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_unshifted_codepoint(event: *mut c_void) -> u32 {
    unsafe { key_event_get_unshifted_codepoint_impl(event) }
}

pub(crate) unsafe fn key_event_get_unshifted_codepoint_impl(event: *mut c_void) -> u32 {
    unsafe {
        ptr::read(key_event_field::<u32>(
            event,
            KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET,
        ))
    }
}
