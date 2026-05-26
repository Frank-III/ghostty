use core::ffi::c_int;

use crate::constants::*;
use crate::input::*;
use crate::mouse_geometry::*;
use crate::mouse_types::*;
use crate::simple::*;

pub(crate) unsafe fn write_mouse_sequence(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    out: *mut u8,
) {
    let mut offset = 0usize;
    match format {
        MOUSE_FORMAT_X10 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[M");
            write_byte(out, &mut offset, button_code.wrapping_add(32));
            write_byte(out, &mut offset, (cell.x as u8).wrapping_add(33));
            write_byte(out, &mut offset, (cell.y as u8).wrapping_add(33));
        },
        MOUSE_FORMAT_UTF8 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[M");
            write_byte(out, &mut offset, button_code.wrapping_add(32));
            write_utf8(out, &mut offset, u32::from(cell.x) + 33);
            write_utf8(out, &mut offset, cell.y + 33);
        },
        MOUSE_FORMAT_SGR => unsafe {
            write_bytes(out, &mut offset, b"\x1B[<");
            write_decimal(out, &mut offset, u64::from(button_code));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.x) + 1);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.y) + 1);
            write_mouse_action_suffix(out, &mut offset, action);
        },
        MOUSE_FORMAT_URXVT => unsafe {
            write_bytes(out, &mut offset, b"\x1B[");
            write_decimal(out, &mut offset, u64::from(button_code) + 32);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.x) + 1);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.y) + 1);
            write_bytes(out, &mut offset, b"M");
        },
        MOUSE_FORMAT_SGR_PIXELS => unsafe {
            let pixels = mouse_pos_to_pixels(pos, size);
            write_bytes(out, &mut offset, b"\x1B[<");
            write_decimal(out, &mut offset, u64::from(button_code));
            write_bytes(out, &mut offset, b";");
            write_signed_decimal(out, &mut offset, pixels.x);
            write_bytes(out, &mut offset, b";");
            write_signed_decimal(out, &mut offset, pixels.y);
            write_mouse_action_suffix(out, &mut offset, action);
        },
        _ => {}
    }
}
