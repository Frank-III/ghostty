use core::ffi::c_int;

use crate::focus_encode::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_focus_encode(
    event: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe { focus_encode(event, out, out_len, out_written) }
}
