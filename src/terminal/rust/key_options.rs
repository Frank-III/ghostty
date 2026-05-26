use core::ffi::c_int;

pub(crate) const KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION: c_int = 0;
pub(crate) const KEY_ENCODER_OPT_KEYPAD_KEY_APPLICATION: c_int = 1;
pub(crate) const KEY_ENCODER_OPT_IGNORE_KEYPAD_WITH_NUMLOCK: c_int = 2;
pub(crate) const KEY_ENCODER_OPT_ALT_ESC_PREFIX: c_int = 3;
pub(crate) const KEY_ENCODER_OPT_MODIFY_OTHER_KEYS_STATE_2: c_int = 4;
pub(crate) const KEY_ENCODER_OPT_MACOS_OPTION_AS_ALT: c_int = 6;
pub(crate) const KEY_ENCODER_OPT_BACKARROW_KEY_MODE: c_int = 7;
pub(crate) const OPTION_AS_ALT_FALSE: c_int = 0;
pub(crate) const OPTION_AS_ALT_TRUE: c_int = 1;
pub(crate) const OPTION_AS_ALT_LEFT: c_int = 2;
pub(crate) const OPTION_AS_ALT_RIGHT: c_int = 3;
