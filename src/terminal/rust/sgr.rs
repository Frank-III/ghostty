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
use crate::sgr_basic::*;
use crate::sgr_constants::*;
use crate::sgr_parse::*;
use crate::sgr_underline::*;
use crate::sgr_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_next(
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
                while i < params_len && sgr_sep_is_set(sep_mask, i) {
                    i += 1;
                }
                i = i.saturating_add(1);
                let partial_len = core::cmp::min(i.saturating_sub(start), params_len - start);
                unsafe {
                    ptr::write(idx, i);
                    write_sgr_unknown(result, params, params_len, params.add(start), partial_len);
                }
                return true;
            }
        }
    }

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
        if start + 1 < params_len {
            let mode = unsafe { ptr::read(params.add(start + 1)) };
            if mode == 2 {
                if let Some((next_idx, r, g, b)) = unsafe {
                    parse_sgr_direct_color(params, params_len, sep_mask, start, colon)
                } {
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
        }
    }

    unsafe {
        ptr::write(idx, params_len);
        write_sgr_unknown(result, params, params_len, params.add(start), params_len - start);
    }
    true
}
