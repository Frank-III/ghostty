use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::simple::*;
use crate::style::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encode(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    pos: GhosttyMousePosition,
    tracking_mode: c_int,
    format: c_int,
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    any_button_pressed: bool,
    track_last_cell: bool,
    last_cell_present: bool,
    last_cell_x: u16,
    last_cell_y: u32,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
    next_last_cell_present: *mut bool,
    next_last_cell_x: *mut u16,
    next_last_cell_y: *mut u32,
) -> c_int {
    let size = GhosttyMouseSize {
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    };

    if size.cell_width == 0 || size.cell_height == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(next_last_cell_present, last_cell_present);
        ptr::write(next_last_cell_x, last_cell_x);
        ptr::write(next_last_cell_y, last_cell_y);
    }

    if !mouse_should_report(action, button_present, button, tracking_mode) {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    if action != MOUSE_ACTION_RELEASE && mouse_pos_out_of_viewport(pos, size) {
        if !mouse_event_sends_motion(tracking_mode) || !any_button_pressed {
            unsafe {
                ptr::write(out_written, 0);
            }
            return GHOSTTY_SUCCESS;
        }
    }

    let cell = mouse_pos_to_cell(pos, size);
    if action == MOUSE_ACTION_MOTION
        && format != MOUSE_FORMAT_SGR_PIXELS
        && track_last_cell
        && last_cell_present
        && last_cell_x == cell.x
        && last_cell_y == cell.y
    {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    if track_last_cell {
        unsafe {
            ptr::write(next_last_cell_present, true);
            ptr::write(next_last_cell_x, cell.x);
            ptr::write(next_last_cell_y, cell.y);
        }
    }

    let Some(button_code) =
        mouse_button_code(action, button_present, button, mods, tracking_mode, format)
    else {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    };

    if format == MOUSE_FORMAT_X10 && (cell.x > 222 || cell.y > 222) {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    let Some(required) = mouse_sequence_len(format, action, button_code, cell, pos, size) else {
        return GHOSTTY_INVALID_VALUE;
    };

    unsafe {
        ptr::write(out_written, required);
    }

    if required > 0 && (out.is_null() || out_len < required) {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_mouse_sequence(format, action, button_code, cell, pos, size, out);
    }

    GHOSTTY_SUCCESS
}

pub(crate) fn mouse_should_report(
    action: c_int,
    button_present: bool,
    button: c_int,
    tracking_mode: c_int,
) -> bool {
    match tracking_mode {
        MOUSE_TRACKING_NONE => false,
        MOUSE_TRACKING_X10 => {
            action == MOUSE_ACTION_PRESS
                && button_present
                && (button == MOUSE_BUTTON_LEFT
                    || button == MOUSE_BUTTON_MIDDLE
                    || button == MOUSE_BUTTON_RIGHT)
        }
        MOUSE_TRACKING_NORMAL => action != MOUSE_ACTION_MOTION,
        MOUSE_TRACKING_BUTTON => button_present,
        MOUSE_TRACKING_ANY => true,
        _ => false,
    }
}

pub(crate) fn mouse_event_sends_motion(tracking_mode: c_int) -> bool {
    tracking_mode == MOUSE_TRACKING_BUTTON || tracking_mode == MOUSE_TRACKING_ANY
}

pub(crate) fn mouse_pos_out_of_viewport(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> bool {
    pos.x < 0.0
        || pos.y < 0.0
        || pos.x > size.screen_width as f32
        || pos.y > size.screen_height as f32
}

pub(crate) fn mouse_grid_size(size: GhosttyMouseSize) -> GhosttyMouseCell {
    let terminal_width = size
        .screen_width
        .saturating_sub(size.padding_left.saturating_add(size.padding_right));
    let terminal_height = size
        .screen_height
        .saturating_sub(size.padding_top.saturating_add(size.padding_bottom));
    let columns = nonzero_u32_div(terminal_width, size.cell_width).max(1);
    let rows = nonzero_u32_div(terminal_height, size.cell_height).max(1);

    GhosttyMouseCell {
        x: columns.min(u32::from(u16::MAX)) as u16,
        y: rows,
    }
}

pub(crate) fn mouse_pos_to_cell(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMouseCell {
    let grid = mouse_grid_size(size);
    let term_x = (pos.x - size.padding_left as f32).max(0.0);
    let term_y = (pos.y - size.padding_top as f32).max(0.0);
    let col = (term_x / size.cell_width as f32) as u32;
    let row = (term_y / size.cell_height as f32) as u32;

    GhosttyMouseCell {
        x: col.min(u32::from(grid.x.saturating_sub(1))) as u16,
        y: row.min(grid.y.saturating_sub(1)),
    }
}

pub(crate) fn nonzero_u32_div(numerator: u32, denominator: u32) -> u32 {
    numerator.checked_div(denominator).unwrap_or(0)
}

pub(crate) fn mouse_pos_to_pixels(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMousePixels {
    GhosttyMousePixels {
        x: round_f32_to_i32(pos.x - size.padding_left as f32),
        y: round_f32_to_i32(pos.y - size.padding_top as f32),
    }
}

pub(crate) fn round_f32_to_i32(value: f32) -> i32 {
    if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}

pub(crate) fn mouse_button_code(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
) -> Option<u8> {
    let mut acc = if !button_present {
        3u8
    } else if action == MOUSE_ACTION_RELEASE
        && format != MOUSE_FORMAT_SGR
        && format != MOUSE_FORMAT_SGR_PIXELS
    {
        3u8
    } else {
        match button {
            MOUSE_BUTTON_LEFT => 0,
            MOUSE_BUTTON_MIDDLE => 1,
            MOUSE_BUTTON_RIGHT => 2,
            MOUSE_BUTTON_FOUR => 64,
            MOUSE_BUTTON_FIVE => 65,
            MOUSE_BUTTON_SIX => 66,
            MOUSE_BUTTON_SEVEN => 67,
            MOUSE_BUTTON_EIGHT => 128,
            MOUSE_BUTTON_NINE => 129,
            _ => return None,
        }
    };

    if tracking_mode != MOUSE_TRACKING_X10 {
        if (mods & MOD_SHIFT) != 0 {
            acc = acc.wrapping_add(4);
        }
        if (mods & MOD_ALT) != 0 {
            acc = acc.wrapping_add(8);
        }
        if (mods & MOD_CTRL) != 0 {
            acc = acc.wrapping_add(16);
        }
    }

    if action == MOUSE_ACTION_MOTION {
        acc = acc.wrapping_add(32);
    }

    Some(acc)
}

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
