use core::ptr;

use crate::sgr_attr::*;
use crate::sgr_basic::*;
use crate::sgr_constants::*;
use crate::sgr_write::*;

pub(crate) unsafe fn write_sgr_basic(
    first: u16,
    i: usize,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    if first == 24 {
        unsafe {
            ptr::write(idx, i);
            write_sgr_c_int(result, SGR_UNDERLINE, 0);
        }
        return true;
    }

    if let Some(tag) = basic_sgr_tag(first) {
        unsafe {
            ptr::write(idx, i);
            if tag == SGR_UNDERLINE {
                write_sgr_c_int(result, tag, 1);
            } else {
                write_sgr_empty(result, tag);
            }
        }
        return true;
    }

    false
}
