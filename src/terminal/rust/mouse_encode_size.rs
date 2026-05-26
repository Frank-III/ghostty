use core::ffi::c_int;

use crate::early::*;
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
