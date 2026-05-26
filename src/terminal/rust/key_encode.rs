use core::ffi::c_int;
use core::ptr;

use crate::key_options::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_bool(
    option: c_int,
    value: bool,
    out: *mut bool,
) {
    if out.is_null() {
        return;
    }

    match option {
        KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION
        | KEY_ENCODER_OPT_KEYPAD_KEY_APPLICATION
        | KEY_ENCODER_OPT_IGNORE_KEYPAD_WITH_NUMLOCK
        | KEY_ENCODER_OPT_ALT_ESC_PREFIX
        | KEY_ENCODER_OPT_MODIFY_OTHER_KEYS_STATE_2
        | KEY_ENCODER_OPT_BACKARROW_KEY_MODE => unsafe {
            ptr::write(out, value);
        },
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_option_as_alt(
    option: c_int,
    value: c_int,
    out: *mut c_int,
) {
    if option != KEY_ENCODER_OPT_MACOS_OPTION_AS_ALT || out.is_null() {
        return;
    }

    match value {
        OPTION_AS_ALT_FALSE | OPTION_AS_ALT_TRUE | OPTION_AS_ALT_LEFT | OPTION_AS_ALT_RIGHT => unsafe {
            ptr::write(out, value);
        },
        _ => {}
    }
}

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
    out_macos_option_as_alt: *mut c_int,
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
