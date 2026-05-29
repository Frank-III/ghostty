use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_button::*;
use crate::mouse_button_required::*;
use crate::mouse_commit::*;
use crate::mouse_encode_size::*;
use crate::mouse_geometry::*;
use crate::mouse_last_cell::*;
use crate::mouse_out_written::*;
use crate::mouse_output::*;
use crate::mouse_size::*;
use crate::mouse_suppress::*;
use crate::mouse_types::*;
use crate::mouse_viewport_report::*;
use crate::mouse_write::*;
use crate::mouse_x10::*;
use crate::render::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::{c_int, c_void};
use core::{mem, ptr};

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
    unsafe {
        mouse_encode_impl(
            action,
            button_present,
            button,
            mods,
            pos,
            tracking_mode,
            format,
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            padding_top,
            padding_bottom,
            padding_right,
            padding_left,
            any_button_pressed,
            track_last_cell,
            last_cell_present,
            last_cell_x,
            last_cell_y,
            out,
            out_len,
            out_written,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        )
    }
}

pub(crate) unsafe fn mouse_encode_impl(
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
    let size = match unsafe {
        mouse_prepare_encode_size(
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            padding_top,
            padding_bottom,
            padding_right,
            padding_left,
            last_cell_present,
            last_cell_x,
            last_cell_y,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        )
    } {
        Ok(size) => size,
        Err(err) => return err,
    };

    if let Err(result) = unsafe {
        mouse_report_viewport_or_suppress(
            action,
            button_present,
            button,
            tracking_mode,
            any_button_pressed,
            pos,
            size,
            out_written,
        )
    } {
        return result;
    }

    unsafe {
        mouse_encode_sequence_after_gate(
            action,
            button_present,
            button,
            mods,
            tracking_mode,
            format,
            pos,
            size,
            track_last_cell,
            last_cell_present,
            last_cell_x,
            last_cell_y,
            out,
            out_len,
            out_written,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        )
    }
}
