use core::ffi::c_int;
use core::ptr;
use crate::early::*;
use crate::constants::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_new(cols: u16, rows: u16) -> c_int {
    if cols == 0 || rows == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_reset(has_terminal: bool) -> bool {
    has_terminal
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_point_from_grid_ref(
    has_point: bool,
    coord: GhosttyPointCoordinate,
    out: *mut GhosttyPointCoordinate,
) -> c_int {
    if !has_point {
        return GHOSTTY_NO_VALUE;
    }

    if !out.is_null() {
        unsafe {
            ptr::write(out, coord);
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_point_from_grid_ref_input(
    has_terminal: bool,
    has_ref: bool,
) -> c_int {
    if !has_terminal || !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

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
    if !has_terminal || cols == 0 || rows == 0 || out_width_px.is_null() || out_height_px.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let width = (u64::from(cols) * u64::from(cell_width_px)).min(u64::from(u32::MAX)) as u32;
    let height = (u64::from(rows) * u64::from(cell_height_px)).min(u64::from(u32::MAX)) as u32;

    unsafe {
        ptr::write(out_width_px, width);
        ptr::write(out_height_px, height);
    }

    GHOSTTY_SUCCESS
}
