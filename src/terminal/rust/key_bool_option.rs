use core::ffi::c_int;

use crate::key_options::*;

pub(crate) fn key_bool_option(option: c_int) -> bool {
    matches!(
        option,
        KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION
            | KEY_ENCODER_OPT_KEYPAD_KEY_APPLICATION
            | KEY_ENCODER_OPT_IGNORE_KEYPAD_WITH_NUMLOCK
            | KEY_ENCODER_OPT_ALT_ESC_PREFIX
            | KEY_ENCODER_OPT_MODIFY_OTHER_KEYS_STATE_2
            | KEY_ENCODER_OPT_BACKARROW_KEY_MODE
    )
}
