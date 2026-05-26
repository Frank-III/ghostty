use core::ffi::{c_int, c_void};
use core::ptr;

use crate::osc::*;

pub(crate) unsafe fn osc_command_data_string(
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
