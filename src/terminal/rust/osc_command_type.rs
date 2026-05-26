use core::ffi::c_int;

use crate::osc::*;

pub(crate) fn osc_command_type(has_command: bool, kind: c_int) -> c_int {
    if has_command {
        kind
    } else {
        OSC_COMMAND_INVALID
    }
}
