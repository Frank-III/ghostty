use core::ffi::{c_int, c_void};

use crate::constants::*;
use crate::early::*;
use crate::kitty_placement_get::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_get(
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
        kitty_placement_get_write(
            data,
            image_id,
            placement_id,
            is_virtual,
            x_offset,
            y_offset,
            source_x,
            source_y,
            source_width,
            source_height,
            columns,
            rows,
            z,
            out,
        )
    }
}
