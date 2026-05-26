use core::ffi::{c_int, c_void};

use crate::early::*;
use crate::osc_command_data::*;
use crate::osc_command_type::*;

pub(crate) const OSC_COMMAND_INVALID: c_int = 0;
pub(crate) const OSC_DATA_CHANGE_WINDOW_TITLE_STR: c_int = 1;

#[no_mangle]
pub extern "C" fn ghostty_rust_osc_command_type(has_command: bool, kind: c_int) -> c_int {
    osc_command_type(has_command, kind)
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_osc_command_data_string(
    data: c_int,
    has_value: bool,
    value: *const u8,
    out: *mut c_void,
) -> bool {
    unsafe { osc_command_data_string(data, has_value, value, out) }
}
