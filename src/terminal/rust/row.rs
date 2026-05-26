use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_row_get(row: u64, data: c_int, out: *mut c_void) -> c_int {
    unsafe { row_get(row, data, out) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_row_get_multi(
    row: u64,
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe { row_get(row, key, out) };
        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, i);
                }
            }
            return result;
        }

        i += 1;
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn row_get(row: u64, data: c_int, out: *mut c_void) -> c_int {
    match data {
        ROW_DATA_WRAP => unsafe { write_out(out, row_bit(row, 32)) },
        ROW_DATA_WRAP_CONTINUATION => unsafe { write_out(out, row_bit(row, 33)) },
        ROW_DATA_GRAPHEME => unsafe { write_out(out, row_bit(row, 34)) },
        ROW_DATA_STYLED => unsafe { write_out(out, row_bit(row, 35)) },
        ROW_DATA_HYPERLINK => unsafe { write_out(out, row_bit(row, 36)) },
        ROW_DATA_SEMANTIC_PROMPT => unsafe { write_out(out, ((row >> 37) & 0x3) as c_int) },
        ROW_DATA_KITTY_VIRTUAL_PLACEHOLDER => unsafe { write_out(out, row_bit(row, 39)) },
        ROW_DATA_DIRTY => unsafe { write_out(out, row_bit(row, 40)) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

pub(crate) fn row_bit(row: u64, bit: u32) -> bool {
    ((row >> bit) & 1) != 0
}
