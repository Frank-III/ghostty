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
use crate::sgr_write::*;

const SGR_UNSET: c_int = 0;
const SGR_UNKNOWN: c_int = 1;
const SGR_BOLD: c_int = 2;
const SGR_RESET_BOLD: c_int = 3;
const SGR_ITALIC: c_int = 4;
const SGR_RESET_ITALIC: c_int = 5;
const SGR_FAINT: c_int = 6;
const SGR_UNDERLINE: c_int = 7;
const SGR_UNDERLINE_COLOR: c_int = 8;
const SGR_256_UNDERLINE_COLOR: c_int = 9;
const SGR_RESET_UNDERLINE_COLOR: c_int = 10;
const SGR_OVERLINE: c_int = 11;
const SGR_RESET_OVERLINE: c_int = 12;
const SGR_BLINK: c_int = 13;
const SGR_RESET_BLINK: c_int = 14;
const SGR_INVERSE: c_int = 15;
const SGR_RESET_INVERSE: c_int = 16;
const SGR_INVISIBLE: c_int = 17;
const SGR_RESET_INVISIBLE: c_int = 18;
const SGR_STRIKETHROUGH: c_int = 19;
const SGR_RESET_STRIKETHROUGH: c_int = 20;
const SGR_DIRECT_COLOR_FG: c_int = 21;
const SGR_DIRECT_COLOR_BG: c_int = 22;
const SGR_8_BG: c_int = 23;
const SGR_8_FG: c_int = 24;
const SGR_RESET_FG: c_int = 25;
const SGR_RESET_BG: c_int = 26;
const SGR_8_BRIGHT_BG: c_int = 27;
const SGR_8_BRIGHT_FG: c_int = 28;
const SGR_256_BG: c_int = 29;
const SGR_256_FG: c_int = 30;

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

fn sgr_sep_is_set(mask: u32, idx: usize) -> bool {
    idx < 32 && (mask & (1u32 << idx)) != 0
}

fn consume_unknown_colon(params_len: usize, sep_mask: u32, idx: usize) -> usize {
    let mut i = idx;
    while i < params_len - 1 && sgr_sep_is_set(sep_mask, i) {
        i += 1;
    }
    i + 1
}

unsafe fn parse_sgr_direct_color(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    start: usize,
    colon: bool,
) -> Option<(usize, u8, u8, u8)> {
    if start + 4 >= params_len {
        return None;
    }
    if !colon {
        unsafe {
            return Some((
                start + 5,
                ptr::read(params.add(start + 2)) as u8,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
            ));
        }
    }

    let count = count_sgr_colon(params_len, sep_mask, start + 1);
    unsafe {
        match count {
            3 => Some((
                start + 5,
                ptr::read(params.add(start + 2)) as u8,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
            )),
            4 if start + 5 < params_len => Some((
                start + 6,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
                ptr::read(params.add(start + 5)) as u8,
            )),
            _ => None,
        }
    }
}

fn count_sgr_colon(params_len: usize, sep_mask: u32, idx: usize) -> usize {
    let mut count = 0usize;
    let mut i = idx;
    while i < params_len - 1 && sgr_sep_is_set(sep_mask, i) {
        count += 1;
        i += 1;
    }
    count
}

unsafe fn write_sgr_unknown(
    result: *mut GhosttySgrAttribute,
    full_ptr: *const u16,
    full_len: usize,
    partial_ptr: *const u16,
    partial_len: usize,
) {
    unsafe {
        write_sgr_empty(result, SGR_UNKNOWN);
        ptr::write(
            core::ptr::addr_of_mut!((*result).value.unknown),
            GhosttySgrUnknown {
                full_ptr,
                full_len,
                partial_ptr,
                partial_len,
            },
        );
    }
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
