use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_button::*;
use crate::mouse_geometry::*;
use crate::mouse_write::*;
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
