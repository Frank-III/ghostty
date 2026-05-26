use core::ffi::c_int;
use core::ptr;

use crate::early::*;
use crate::focus_sequence::*;
use crate::focus_write::*;

pub(crate) unsafe fn focus_encode(
    event: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let seq = focus_sequence(event);

    unsafe {
        ptr::write(out_written, seq.len());
    }

    if out.is_null() || out_len < seq.len() {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_focus_sequence(out, seq);
    }

    GHOSTTY_SUCCESS
}
