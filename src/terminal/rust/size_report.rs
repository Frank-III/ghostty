use core::ffi::c_int;

use crate::constants::*;
use crate::size_report_encode::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_size_report_encode(
    style: c_int,
    size: GhosttySizeReportSize,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { size_report_encode(style, size, out, out_len, out_written) }
}
