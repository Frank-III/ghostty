use core::ffi::c_int;

use crate::early::*;
use crate::mouse_last_cell::*;
use crate::mouse_size::*;
use crate::mouse_types::*;

pub(crate) fn mouse_encode_size(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
) -> Result<GhosttyMouseSize, c_int> {
    let size = mouse_size_from_parts(
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    );

    if !mouse_size_has_cell_size(size) {
        return Err(GHOSTTY_INVALID_VALUE);
    }

    Ok(size)
}

pub(crate) unsafe fn mouse_prepare_encode_size(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    last_cell_present: bool,
    last_cell_x: u16,
    last_cell_y: u32,
    next_last_cell_present: *mut bool,
    next_last_cell_x: *mut u16,
    next_last_cell_y: *mut u32,
) -> Result<GhosttyMouseSize, c_int> {
    let size = mouse_encode_size(
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    )?;

    unsafe {
        mouse_carry_forward_last_cell(
            last_cell_present,
            last_cell_x,
            last_cell_y,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        );
    }

    Ok(size)
}
