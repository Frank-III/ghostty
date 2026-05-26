use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;

pub(crate) unsafe fn kitty_placement_get_write(
    data: c_int,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match data {
            KITTY_PLACEMENT_DATA_IMAGE_ID => ptr::write(out.cast::<u32>(), image_id),
            KITTY_PLACEMENT_DATA_PLACEMENT_ID => ptr::write(out.cast::<u32>(), placement_id),
            KITTY_PLACEMENT_DATA_IS_VIRTUAL => ptr::write(out.cast::<bool>(), is_virtual),
            KITTY_PLACEMENT_DATA_X_OFFSET => ptr::write(out.cast::<u32>(), x_offset),
            KITTY_PLACEMENT_DATA_Y_OFFSET => ptr::write(out.cast::<u32>(), y_offset),
            KITTY_PLACEMENT_DATA_SOURCE_X => ptr::write(out.cast::<u32>(), source_x),
            KITTY_PLACEMENT_DATA_SOURCE_Y => ptr::write(out.cast::<u32>(), source_y),
            KITTY_PLACEMENT_DATA_SOURCE_WIDTH => ptr::write(out.cast::<u32>(), source_width),
            KITTY_PLACEMENT_DATA_SOURCE_HEIGHT => ptr::write(out.cast::<u32>(), source_height),
            KITTY_PLACEMENT_DATA_COLUMNS => ptr::write(out.cast::<u32>(), columns),
            KITTY_PLACEMENT_DATA_ROWS => ptr::write(out.cast::<u32>(), rows),
            KITTY_PLACEMENT_DATA_Z => ptr::write(out.cast::<i32>(), z),
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}
