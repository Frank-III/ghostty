use core::ffi::c_int;

use crate::early::*;
use crate::mouse_button_required::*;
use crate::mouse_last_cell::*;
use crate::mouse_output::*;
use crate::mouse_sequence_len::*;
use crate::mouse_types::*;
use crate::mouse_write::*;
use crate::mouse_x10::*;

pub(crate) unsafe fn mouse_commit_sequence(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    required: usize,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe {
        mouse_commit_output_len(out_written, required);
    }

    if mouse_output_needs_space(required, out, out_len) {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        mouse_output_sequence(format, action, button_code, cell, pos, size, out);
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn mouse_finalize_sequence(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
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

pub(crate) unsafe fn mouse_encode_sequence_after_gate(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
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
    let cell = match unsafe {
        mouse_cell_or_suppress_same_cell_motion(
            action,
            format,
            pos,
            size,
            track_last_cell,
            last_cell_present,
            last_cell_x,
            last_cell_y,
            out_written,
            next_last_cell_present,
            next_last_cell_x,
            next_last_cell_y,
        )
    } {
        Ok(cell) => cell,
        Err(result) => return result,
    };

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

    unsafe {
        mouse_finalize_sequence(
            format,
            action,
            button_code,
            cell,
            pos,
            size,
            out,
            out_len,
            out_written,
        )
    }
}
