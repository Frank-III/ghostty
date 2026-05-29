use core::ptr;

use crate::sgr_attr::*;
use crate::sgr_constants::*;
use crate::sgr_parse::*;
use crate::sgr_write::*;

pub(crate) unsafe fn write_sgr_color(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    start: usize,
    first: u16,
    colon: bool,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    if start + 1 >= params_len {
        return false;
    }

    let mode = unsafe { ptr::read(params.add(start + 1)) };
    if mode == 2 {
        if let Some((next_idx, r, g, b)) =
            unsafe { parse_sgr_direct_color(params, params_len, sep_mask, start, colon) }
        {
            unsafe {
                ptr::write(idx, next_idx);
                write_sgr_rgb(
                    result,
                    match first {
                        38 => SGR_DIRECT_COLOR_FG,
                        48 => SGR_DIRECT_COLOR_BG,
                        _ => SGR_UNDERLINE_COLOR,
                    },
                    r,
                    g,
                    b,
                );
            }
            return true;
        }
    } else if mode == 5 && start + 2 < params_len {
        let value = unsafe { ptr::read(params.add(start + 2)) as u8 };
        unsafe {
            ptr::write(idx, start + 3);
            write_sgr_u8(
                result,
                match first {
                    38 => SGR_256_FG,
                    48 => SGR_256_BG,
                    _ => SGR_256_UNDERLINE_COLOR,
                },
                value,
            );
        }
        return true;
    }

    false
}
