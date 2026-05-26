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
use crate::sgr_constants::*;
use crate::sgr_parse::*;
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

    let tag = match first {
        0 => Some(SGR_UNSET),
        1 => Some(SGR_BOLD),
        2 => Some(SGR_FAINT),
        3 => Some(SGR_ITALIC),
        5 | 6 => Some(SGR_BLINK),
        7 => Some(SGR_INVERSE),
        8 => Some(SGR_INVISIBLE),
        9 => Some(SGR_STRIKETHROUGH),
        21 => Some(SGR_UNDERLINE),
        22 => Some(SGR_RESET_BOLD),
        23 => Some(SGR_RESET_ITALIC),
        24 => {
            unsafe {
                ptr::write(idx, i);
                write_sgr_c_int(result, SGR_UNDERLINE, 0);
            }
            return true;
        }
        25 => Some(SGR_RESET_BLINK),
        27 => Some(SGR_RESET_INVERSE),
        28 => Some(SGR_RESET_INVISIBLE),
        29 => Some(SGR_RESET_STRIKETHROUGH),
        39 => Some(SGR_RESET_FG),
        49 => Some(SGR_RESET_BG),
        53 => Some(SGR_OVERLINE),
        55 => Some(SGR_RESET_OVERLINE),
        59 => Some(SGR_RESET_UNDERLINE_COLOR),
        _ => None,
    };

    if let Some(tag) = tag {
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

    if first >= 30 && first <= 37 {
        unsafe {
            ptr::write(idx, i);
            write_sgr_c_int(result, SGR_8_FG, c_int::from(first - 30));
        }
        return true;
    }
    if first >= 40 && first <= 47 {
        unsafe {
            ptr::write(idx, i);
            write_sgr_c_int(result, SGR_8_BG, c_int::from(first - 40));
        }
        return true;
    }
    if first >= 90 && first <= 97 {
        unsafe {
            ptr::write(idx, i);
            write_sgr_c_int(result, SGR_8_BRIGHT_FG, c_int::from(first - 82));
        }
        return true;
    }
    if first >= 100 && first <= 107 {
        unsafe {
            ptr::write(idx, i);
            write_sgr_c_int(result, SGR_8_BRIGHT_BG, c_int::from(first - 92));
        }
        return true;
    }

    if first == 4 && colon {
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
        return true;
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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_parser_reset(idx: *mut usize) {
    if idx.is_null() {
        return;
    }

    unsafe {
        ptr::write(idx, 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_params_sep_mask(seps: *const u8, len: usize) -> u32 {
    if seps.is_null() {
        return 0;
    }

    let mut mask = 0u32;
    let mut i = 0usize;
    while i < len && i < 24 {
        let sep = unsafe { ptr::read(seps.add(i)) };
        if sep == b':' {
            mask |= 1u32 << i;
        }
        i += 1;
    }
    mask
}
