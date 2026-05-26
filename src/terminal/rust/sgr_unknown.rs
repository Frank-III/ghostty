use core::ptr;

use crate::sgr_attr::*;
use crate::sgr_parse::*;
use crate::sgr_write::*;

pub(crate) unsafe fn write_sgr_unknown_colon(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    start: usize,
    i: usize,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    let mut next = i;
    while next < params_len && sgr_sep_is_set(sep_mask, next) {
        next += 1;
    }
    next = next.saturating_add(1);

    let partial_len = core::cmp::min(next.saturating_sub(start), params_len - start);
    unsafe {
        ptr::write(idx, next);
        write_sgr_unknown(result, params, params_len, params.add(start), partial_len);
    }
    true
}

pub(crate) unsafe fn write_sgr_unknown_rest(
    params: *const u16,
    params_len: usize,
    start: usize,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    unsafe {
        ptr::write(idx, params_len);
        write_sgr_unknown(result, params, params_len, params.add(start), params_len - start);
    }
    true
}
