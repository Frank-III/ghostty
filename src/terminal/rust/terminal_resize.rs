use core::ffi::c_int;

use crate::early::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_resize(
    has_terminal: bool,
    cols: u16,
    rows: u16,
    cell_width_px: u32,
    cell_height_px: u32,
    out_width_px: *mut u32,
    out_height_px: *mut u32,
) -> c_int {
    unsafe {
        terminal_resize_impl(
            has_terminal,
            cols,
            rows,
            cell_width_px,
            cell_height_px,
            out_width_px,
            out_height_px,
        )
    }
}

pub(crate) unsafe fn terminal_resize_impl(
    has_terminal: bool,
    cols: u16,
    rows: u16,
    cell_width_px: u32,
    cell_height_px: u32,
    out_width_px: *mut u32,
    out_height_px: *mut u32,
) -> c_int {
    if !has_terminal || cols == 0 || rows == 0 || out_width_px.is_null() || out_height_px.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let width = (u64::from(cols) * u64::from(cell_width_px)).min(u64::from(u32::MAX)) as u32;
    let height = (u64::from(rows) * u64::from(cell_height_px)).min(u64::from(u32::MAX)) as u32;

    unsafe {
        core::ptr::write(out_width_px, width);
        core::ptr::write(out_height_px, height);
    }

    GHOSTTY_SUCCESS
}
