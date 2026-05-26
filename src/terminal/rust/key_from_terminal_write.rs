use core::ffi::c_int;
use core::ptr;

use crate::key_options::*;

pub(crate) unsafe fn key_write_ptr<T>(out: *mut T, value: T) {
    unsafe {
        ptr::write(out, value);
    }
}

pub(crate) unsafe fn key_write_ptr_if_present<T>(out: *mut T, value: T) {
    if out.is_null() {
        return;
    }

    unsafe {
        key_write_ptr(out, value);
    }
}

pub(crate) unsafe fn key_from_terminal_write(
    alt_esc_prefix: bool,
    cursor_key_application: bool,
    keypad_key_application: bool,
    backarrow_key_mode: bool,
    ignore_keypad_with_numlock: bool,
    modify_other_keys_state_2: bool,
    out_alt_esc_prefix: *mut bool,
    out_cursor_key_application: *mut bool,
    out_keypad_key_application: *mut bool,
    out_backarrow_key_mode: *mut bool,
    out_ignore_keypad_with_numlock: *mut bool,
    out_modify_other_keys_state_2: *mut bool,
    out_macos_option_as_alt: *mut c_int,
) {
    unsafe {
        key_write_ptr(out_alt_esc_prefix, alt_esc_prefix);
        key_write_ptr(out_cursor_key_application, cursor_key_application);
        key_write_ptr(out_keypad_key_application, keypad_key_application);
        key_write_ptr(out_backarrow_key_mode, backarrow_key_mode);
        key_write_ptr(out_ignore_keypad_with_numlock, ignore_keypad_with_numlock);
        key_write_ptr(out_modify_other_keys_state_2, modify_other_keys_state_2);
        key_write_ptr(out_macos_option_as_alt, OPTION_AS_ALT_FALSE);
    }
}
