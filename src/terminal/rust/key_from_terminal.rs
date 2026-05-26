use crate::key_from_terminal_write::key_from_terminal_write;

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
        key_from_terminal_write(
            alt_esc_prefix,
            cursor_key_application,
            keypad_key_application,
            backarrow_key_mode,
            ignore_keypad_with_numlock,
            modify_other_keys_state_2,
            out_alt_esc_prefix,
            out_cursor_key_application,
            out_keypad_key_application,
            out_backarrow_key_mode,
            out_ignore_keypad_with_numlock,
            out_modify_other_keys_state_2,
            out_macos_option_as_alt,
        );
    }
}
