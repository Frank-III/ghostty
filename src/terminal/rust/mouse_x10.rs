use crate::constants::*;
use crate::mouse_suppress::*;
use crate::mouse_types::*;

pub(crate) fn mouse_x10_cell_in_bounds(format: core::ffi::c_int, cell: GhosttyMouseCell) -> bool {
    format != MOUSE_FORMAT_X10 || (cell.x <= 222 && cell.y <= 222)
}

pub(crate) unsafe fn mouse_x10_bounds_or_suppress(
    format: core::ffi::c_int,
    cell: GhosttyMouseCell,
    out_written: *mut usize,
) -> Result<(), core::ffi::c_int> {
    if mouse_x10_cell_in_bounds(format, cell) {
        Ok(())
    } else {
        Err(unsafe { mouse_suppress_result(out_written) })
    }
}
