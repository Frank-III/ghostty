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
use crate::mouse_button_required::*;
use crate::mouse_commit::*;
use crate::mouse_encode_size::*;
use crate::mouse_geometry::*;
use crate::mouse_last_cell::*;
use crate::mouse_output::*;
use crate::mouse_out_written::*;
use crate::mouse_size::*;
use crate::mouse_suppress::*;
use crate::mouse_types::*;
use crate::mouse_viewport_report::*;
use crate::mouse_write::*;
use crate::mouse_x10::*;
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
    let size = match mouse_encode_size(
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    ) {
        Ok(size) => size,
        Err(err) => return err,
    };

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

    if !mouse_should_report(action, button_present, button, tracking_mode) {
        return unsafe { mouse_suppress_result(out_written) };
    }

    if let Err(result) = unsafe {
        mouse_viewport_or_suppress(
            action,
            tracking_mode,
            any_button_pressed,
            pos,
            size,
            out_written,
        )
    } {
        return result;
    }

    let cell = mouse_pos_to_cell(pos, size);
    if mouse_should_suppress_same_cell_motion(
        action,
        format,
        track_last_cell,
        last_cell_present,
        last_cell_x,
        last_cell_y,
        cell,
    ) {
        return unsafe { mouse_suppress_result(out_written) };
    }

    unsafe {
        mouse_update_tracked_last_cell(
            track_last_cell,
            cell,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        );
    }

    let button_code = match unsafe {
        mouse_required_button_code_or_suppress(
            action,
            button_present,
            button,
            mods,
            tracking_mode,
            format,
            out_written,
        )
    } {
        Ok(button_code) => button_code,
        Err(result) => return result,
    };

    if let Err(result) = unsafe { mouse_x10_bounds_or_suppress(format, cell, out_written) } {
        return result;
    }

    let required = match mouse_required_sequence_len(format, action, button_code, cell, pos, size) {
        Ok(required) => required,
        Err(err) => return err,
    };

    unsafe {
        mouse_commit_sequence(
            format,
            action,
            button_code,
            cell,
            pos,
            size,
            required,
            out,
            out_len,
            out_written,
        )
    }
}
