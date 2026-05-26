use core::ffi::c_int;

use crate::early::*;

pub(crate) const FOCUS_GAINED: &[u8; 3] = b"\x1B[I";
pub(crate) const FOCUS_LOST: &[u8; 3] = b"\x1B[O";

pub(crate) fn focus_sequence(event: c_int) -> &'static [u8; 3] {
    if event == GHOSTTY_FOCUS_LOST {
        FOCUS_LOST
    } else {
        FOCUS_GAINED
    }
}
