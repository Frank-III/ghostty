use core::ffi::c_int;

use crate::early::*;
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
