use core::ffi::c_int;

use crate::key_options::*;

pub(crate) fn key_option_as_alt_value(value: c_int) -> bool {
    matches!(
        value,
        OPTION_AS_ALT_FALSE | OPTION_AS_ALT_TRUE | OPTION_AS_ALT_LEFT | OPTION_AS_ALT_RIGHT
    )
}
