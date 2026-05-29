use crate::cell::*;
use crate::constants::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::render_row_data::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::{c_int, c_void};
use core::{mem, ptr};

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_get(
    has_iterator: bool,
    has_row: bool,
    data: c_int,
    has_out: bool,
    out_size: usize,
) -> c_int {
    render_row_get_impl(has_iterator, has_row, data, has_out, out_size)
}

pub(crate) fn render_row_get_impl(
    has_iterator: bool,
    has_row: bool,
    data: c_int,
    has_out: bool,
    out_size: usize,
) -> c_int {
    if !has_iterator || !has_row || !has_out {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match data {
        RENDER_ROW_DATA_DIRTY | RENDER_ROW_DATA_RAW | RENDER_ROW_DATA_CELLS => {
            RENDER_RESULT_SUCCESS
        }
        RENDER_ROW_DATA_SELECTION => {
            if out_size < mem::size_of::<GhosttyRenderRowSelection>() {
                RENDER_RESULT_INVALID_VALUE
            } else {
                RENDER_RESULT_SUCCESS
            }
        }
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    raw: u64,
    dirty: bool,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
) -> c_int {
    unsafe {
        render_row_get_multi_impl(
            count,
            keys,
            values,
            out_written,
            raw,
            dirty,
            selection_present,
            selection_start,
            selection_end,
        )
    }
}

pub(crate) unsafe fn render_row_get_multi_impl(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    raw: u64,
    dirty: bool,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let out_size = if key == RENDER_ROW_DATA_SELECTION && !out.is_null() {
            unsafe {
                let selection = out.cast::<GhosttyRenderRowSelection>();
                ptr::read(core::ptr::addr_of!((*selection).size))
            }
        } else {
            0
        };

        let result = unsafe {
            ghostty_rust_render_row_get_data(
                key,
                raw,
                dirty,
                selection_present,
                selection_start,
                selection_end,
                out_size,
                out,
            )
        };

        if result != RENDER_RESULT_SUCCESS {
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

    RENDER_RESULT_SUCCESS
}
