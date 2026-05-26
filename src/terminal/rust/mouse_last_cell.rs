use core::ptr;

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
