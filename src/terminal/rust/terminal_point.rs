use core::ffi::c_int;
use core::ptr;

use crate::early::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_point_from_grid_ref(
    has_point: bool,
    coord: GhosttyPointCoordinate,
    out: *mut GhosttyPointCoordinate,
) -> c_int {
    unsafe { terminal_point_from_grid_ref_impl(has_point, coord, out) }
}

pub(crate) unsafe fn terminal_point_from_grid_ref_impl(
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
    terminal_point_from_grid_ref_input_impl(has_terminal, has_ref)
}

pub(crate) fn terminal_point_from_grid_ref_input_impl(
    has_terminal: bool,
    has_ref: bool,
) -> c_int {
    if !has_terminal || !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}
