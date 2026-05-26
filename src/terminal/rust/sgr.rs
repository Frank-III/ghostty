use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;
use crate::color::*;
use crate::allocator::*;
use crate::sgr_attr::*;
use crate::sgr_8color::*;
use crate::sgr_basic_write::*;
use crate::sgr_color::*;
use crate::sgr_constants::*;
use crate::sgr_parse::*;
use crate::sgr_underline::*;
use crate::sgr_unknown::*;
use crate::sgr_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_next(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    unsafe { sgr_next_impl(params, params_len, sep_mask, idx, result) }
}

pub(crate) unsafe fn sgr_next_impl(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    idx: *mut usize,
    result: *mut GhosttySgrAttribute,
) -> bool {
    if idx.is_null() || result.is_null() {
        return false;
    }

    let mut i = unsafe { ptr::read(idx) };
    if i >= params_len {
        unsafe {
            ptr::write(idx, i.saturating_add(1));
        }
        if i == 0 {
            unsafe { write_sgr_empty(result, SGR_UNSET) };
            return true;
        }
        return false;
    }

    let start = i;
    let first = unsafe { ptr::read(params.add(i)) };
    let colon = sgr_sep_is_set(sep_mask, i);
    i += 1;

    if colon {
        match first {
            4 | 38 | 48 | 58 => {}
            _ => {
                return unsafe {
                    write_sgr_unknown_colon(params, params_len, sep_mask, start, i, idx, result)
                };
            }
        }
    }

    if unsafe { write_sgr_basic(first, i, idx, result) } {
        return true;
    }

    if unsafe { write_sgr_8_color(result, first) } {
        unsafe {
            ptr::write(idx, i);
        }
        return true;
    }

    if first == 4 && colon {
        return unsafe {
            write_sgr_colon_underline(params, params_len, sep_mask, start, i, idx, result)
        };
    }

    if first == 38 || first == 48 || first == 58 {
        if unsafe { write_sgr_color(params, params_len, sep_mask, start, first, colon, idx, result) } {
            return true;
        }
    }

    unsafe { write_sgr_unknown_rest(params, params_len, start, idx, result) }
}
