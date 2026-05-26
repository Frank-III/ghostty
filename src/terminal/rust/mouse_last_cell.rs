use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::mouse_types::*;

pub(crate) unsafe fn mouse_write_last_cell(
    present: bool,
    x: u16,
    y: u32,
    out_present: *mut bool,
    out_x: *mut u16,
    out_y: *mut u32,
) {
    unsafe {
        ptr::write(out_present, present);
        ptr::write(out_x, x);
        ptr::write(out_y, y);
    }
}

pub(crate) unsafe fn mouse_carry_forward_last_cell(
    last_cell_present: bool,
    last_cell_x: u16,
    last_cell_y: u32,
    out_present: *mut bool,
    out_x: *mut u16,
    out_y: *mut u32,
) {
    unsafe {
        mouse_write_last_cell(
            last_cell_present,
            last_cell_x,
            last_cell_y,
            out_present,
            out_x,
            out_y,
        );
    }
}

pub(crate) unsafe fn mouse_write_last_cell_from_cell(
    cell: GhosttyMouseCell,
    out_present: *mut bool,
    out_x: *mut u16,
    out_y: *mut u32,
) {
    unsafe {
        mouse_write_last_cell(true, cell.x, cell.y, out_present, out_x, out_y);
    }
}

pub(crate) fn mouse_should_suppress_same_cell_motion(
    action: c_int,
    format: c_int,
    track_last_cell: bool,
    last_cell_present: bool,
    last_cell_x: u16,
    last_cell_y: u32,
    cell: GhosttyMouseCell,
) -> bool {
    action == MOUSE_ACTION_MOTION
        && format != MOUSE_FORMAT_SGR_PIXELS
        && track_last_cell
        && last_cell_present
        && last_cell_x == cell.x
        && last_cell_y == cell.y
}

pub(crate) unsafe fn mouse_update_tracked_last_cell(
    track_last_cell: bool,
    cell: GhosttyMouseCell,
    out_present: *mut bool,
    out_x: *mut u16,
    out_y: *mut u32,
) {
    if track_last_cell {
        unsafe {
            mouse_write_last_cell_from_cell(cell, out_present, out_x, out_y);
        }
    }
}
