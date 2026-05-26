use core::ptr;

use crate::sgr_attr::*;
use crate::sgr_constants::*;
use crate::sgr_parse::*;
use crate::sgr_write::*;

pub(crate) unsafe fn write_sgr_colon_underline(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    start: usize,
    i: usize,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    if start + 1 < params_len && !sgr_sep_is_set(sep_mask, start + 1) {
        let style = unsafe { ptr::read(params.add(start + 1)) };
        unsafe {
            ptr::write(idx, start + 2);
            write_sgr_c_int(
                result,
                SGR_UNDERLINE,
                match style {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 3,
                    4 => 4,
                    5 => 5,
                    _ => 1,
                },
            );
        }
        return true;
    }

    let next_idx = consume_unknown_colon(params_len, sep_mask, i);
    unsafe {
        ptr::write(idx, next_idx);
        write_sgr_unknown(
            result,
            params,
            params_len,
            params.add(start),
            core::cmp::min(next_idx.saturating_sub(start), params_len - start),
        );
    }
    true
}
