use core::ptr;

use crate::key_options::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_from_terminal(
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
    out_macos_option_as_alt: *mut core::ffi::c_int,
) {
    unsafe {
        ptr::write(out_alt_esc_prefix, alt_esc_prefix);
        ptr::write(out_cursor_key_application, cursor_key_application);
        ptr::write(out_keypad_key_application, keypad_key_application);
        ptr::write(out_backarrow_key_mode, backarrow_key_mode);
        ptr::write(out_ignore_keypad_with_numlock, ignore_keypad_with_numlock);
        ptr::write(out_modify_other_keys_state_2, modify_other_keys_state_2);
        ptr::write(out_macos_option_as_alt, OPTION_AS_ALT_FALSE);
    }
}
