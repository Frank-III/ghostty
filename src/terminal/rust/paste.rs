use core::ffi::c_int;

use crate::paste_encode::*;
use crate::paste_is_safe::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_is_safe(data: *const u8, len: usize) -> bool {
    unsafe { paste_is_safe_impl(data, len) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_encode(
    data: *mut u8,
    data_len: usize,
    bracketed: bool,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { paste_encode_impl(data, data_len, bracketed, out, out_len, out_written) }
}

pub(crate) unsafe fn paste_is_safe_impl(data: *const u8, len: usize) -> bool {
    unsafe { paste_is_safe(data, len) }
}

pub(crate) unsafe fn paste_encode_impl(
    data: *mut u8,
    data_len: usize,
    bracketed: bool,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { paste_encode(data, data_len, bracketed, out, out_len, out_written) }
}
