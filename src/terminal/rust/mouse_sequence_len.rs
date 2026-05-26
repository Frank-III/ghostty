use core::ffi::c_int;

use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::mouse_geometry::*;
use crate::mouse_types::*;
use crate::simple::*;

pub(crate) fn mouse_sequence_len(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
) -> Option<usize> {
    match format {
        MOUSE_FORMAT_X10 => Some(b"\x1B[M".len() + 3),
        MOUSE_FORMAT_UTF8 => {
            Some(b"\x1B[M".len() + 1 + utf8_len(u32::from(cell.x) + 33)? + utf8_len(cell.y + 33)?)
        }
        MOUSE_FORMAT_SGR => Some(
            b"\x1B[<".len()
                + decimal_len(u64::from(button_code))
                + 1
                + decimal_len(u64::from(cell.x) + 1)
                + 1
                + decimal_len(u64::from(cell.y) + 1)
                + mouse_action_suffix_len(action),
        ),
        MOUSE_FORMAT_URXVT => Some(
            b"\x1B[".len()
                + decimal_len(u64::from(button_code) + 32)
                + 1
                + decimal_len(u64::from(cell.x) + 1)
                + 1
                + decimal_len(u64::from(cell.y) + 1)
                + 1,
        ),
        MOUSE_FORMAT_SGR_PIXELS => {
            let pixels = mouse_pos_to_pixels(pos, size);
            Some(
                b"\x1B[<".len()
                    + decimal_len(u64::from(button_code))
                    + 1
                    + signed_decimal_len(pixels.x)
                    + 1
                    + signed_decimal_len(pixels.y)
                    + mouse_action_suffix_len(action),
            )
        }
        _ => None,
    }
}

pub(crate) fn mouse_action_suffix_len(_: c_int) -> usize {
    1
}

pub(crate) fn mouse_required_sequence_len(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
) -> Result<usize, c_int> {
    mouse_sequence_len(format, action, button_code, cell, pos, size).ok_or(GHOSTTY_INVALID_VALUE)
}
