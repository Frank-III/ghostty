use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::constants::*;
use crate::early::*;

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
