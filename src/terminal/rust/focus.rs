use core::ffi::c_int;
use core::ptr;

use crate::early::*;

pub(crate) const FOCUS_GAINED: &[u8; 3] = b"\x1B[I";
pub(crate) const FOCUS_LOST: &[u8; 3] = b"\x1B[O";

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_focus_encode(
    event: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let seq = if event == GHOSTTY_FOCUS_LOST {
        FOCUS_LOST
    } else {
        FOCUS_GAINED
    };

    unsafe {
        ptr::write(out_written, seq.len());
    }

    if out.is_null() || out_len < seq.len() {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        ptr::copy_nonoverlapping(seq.as_ptr(), out, seq.len());
    }

    GHOSTTY_SUCCESS
}
