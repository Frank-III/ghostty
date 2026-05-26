use crate::constants::*;
use crate::mouse_types::*;

pub(crate) fn mouse_x10_cell_in_bounds(format: core::ffi::c_int, cell: GhosttyMouseCell) -> bool {
    format != MOUSE_FORMAT_X10 || (cell.x <= 222 && cell.y <= 222)
}
