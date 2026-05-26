use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;
use crate::cell::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_next(
    has_current: bool,
    current: u16,
    len: usize,
    out_next: *mut u16,
) -> bool {
    if out_next.is_null() {
        return false;
    }

    let next = if has_current {
        match current.checked_add(1) {
            Some(value) => value,
            None => return false,
        }
    } else {
        0
    };

    if usize::from(next) >= len {
        return false;
    }

    unsafe {
        ptr::write(out_next, next);
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_select(index: u16, len: usize) -> c_int {
    if usize::from(index) >= len {
        return RENDER_RESULT_INVALID_VALUE;
    }

    RENDER_RESULT_SUCCESS
}

#[repr(C)]
pub struct GhosttyRenderRowSelection {
    pub(crate) size: usize,
    pub(crate) start_x: u16,
    pub(crate) end_x: u16,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_dirty(dirty: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, dirty);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_data(
    data: c_int,
    raw: u64,
    dirty: bool,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    out_size: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_ROW_DATA_DIRTY => ptr::write(out.cast::<bool>(), dirty),
            RENDER_ROW_DATA_RAW => ptr::write(out.cast::<u64>(), raw),
            RENDER_ROW_DATA_SELECTION => {
                if out_size < mem::size_of::<GhosttyRenderRowSelection>() {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                if !selection_present {
                    return GHOSTTY_NO_VALUE;
                }
                let selection = out.cast::<GhosttyRenderRowSelection>();
                ptr::write(
                    core::ptr::addr_of_mut!((*selection).start_x),
                    selection_start,
                );
                ptr::write(core::ptr::addr_of_mut!((*selection).end_x), selection_end);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_selection(
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    out_size: usize,
    out: *mut GhosttyRenderRowSelection,
) -> c_int {
    if out.is_null() || out_size < mem::size_of::<GhosttyRenderRowSelection>() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    if !selection_present {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).start_x), selection_start);
        ptr::write(core::ptr::addr_of_mut!((*out).end_x), selection_end);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_get(
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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_set_dirty(value: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_set(
    has_iterator: bool,
    has_row: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    if !has_iterator || !has_row || !has_value {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match option {
        RENDER_ROW_SET_DIRTY => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}
