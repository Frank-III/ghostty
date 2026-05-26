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
