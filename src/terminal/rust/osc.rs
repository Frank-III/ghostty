use core::ffi::{c_int, c_void};
use core::ptr;

use crate::early::*;

pub(crate) const OSC_COMMAND_INVALID: c_int = 0;
pub(crate) const OSC_DATA_CHANGE_WINDOW_TITLE_STR: c_int = 1;

#[no_mangle]
pub extern "C" fn ghostty_rust_osc_command_type(has_command: bool, kind: c_int) -> c_int {
    if has_command {
        kind
    } else {
        OSC_COMMAND_INVALID
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_osc_command_data_string(
    data: c_int,
    has_value: bool,
    value: *const u8,
    out: *mut c_void,
) -> bool {
    if data != OSC_DATA_CHANGE_WINDOW_TITLE_STR || !has_value || value.is_null() || out.is_null() {
        return false;
    }

    unsafe {
        ptr::write(out.cast::<*const u8>(), value);
    }

    true
}
