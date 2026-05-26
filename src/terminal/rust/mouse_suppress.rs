use core::ffi::c_int;

use crate::early::*;
use crate::mouse_output::*;

pub(crate) unsafe fn mouse_suppress_result(out_written: *mut usize) -> c_int {
    unsafe {
        mouse_suppress_output(out_written);
    }

    GHOSTTY_SUCCESS
}
