use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::size_report_len::*;
use crate::size_report_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_size_report_encode(
    style: c_int,
    size: GhosttySizeReportSize,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let Some(required) = size_report_len(style, size) else {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_INVALID_VALUE;
    };

    unsafe {
        ptr::write(out_written, required);
    }

    if out.is_null() || out_len < required {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_size_report(style, size, out);
    }

    GHOSTTY_SUCCESS
}
