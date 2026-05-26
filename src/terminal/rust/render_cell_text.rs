use core::ffi::{c_int, c_void};
use core::ptr;

use crate::cell::*;
use crate::constants::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_selected(
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    x: u16,
    out: *mut bool,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            out,
            selection_present && x >= selection_start && x <= selection_end,
        );
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_get_text(
    data: c_int,
    cell: u64,
    extra: *const u32,
    extra_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_ROW_CELL_DATA_RAW => ptr::write(out.cast::<u64>(), cell),
            RENDER_ROW_CELL_DATA_GRAPHEMES_LEN => {
                let len = if cell_has_text(cell) {
                    1usize.wrapping_add(if cell_has_grapheme(cell) {
                        extra_len
                    } else {
                        0
                    })
                } else {
                    0
                };
                ptr::write(out.cast::<u32>(), len as u32);
            }
            RENDER_ROW_CELL_DATA_GRAPHEMES_BUF => {
                if !cell_has_text(cell) {
                    return RENDER_RESULT_SUCCESS;
                }

                let buf = out.cast::<u32>();
                ptr::write(buf, cell_codepoint(cell));
                if cell_has_grapheme(cell) {
                    if extra.is_null() && extra_len > 0 {
                        return RENDER_RESULT_INVALID_VALUE;
                    }

                    let mut i = 0usize;
                    while i < extra_len {
                        ptr::write(buf.add(i + 1), ptr::read(extra.add(i)));
                        i += 1;
                    }
                }
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}
