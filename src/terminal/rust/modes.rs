use core::ffi::c_int;

use crate::mode_report_encode::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mode_report_encode(
    tag: u16,
    state: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { mode_report_encode_impl(tag, state, out, out_len, out_written) }
}

pub(crate) unsafe fn mode_report_encode_impl(
    tag: u16,
    state: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { mode_report_encode(tag, state, out, out_len, out_written) }
}
