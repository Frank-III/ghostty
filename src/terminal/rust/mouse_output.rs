use crate::mouse_out_written::*;

pub(crate) unsafe fn mouse_suppress_output(out_written: *mut usize) {
    unsafe {
        mouse_write_out_written(out_written, 0);
    }
}

pub(crate) unsafe fn mouse_commit_output_len(out_written: *mut usize, required: usize) {
    unsafe {
        mouse_write_out_written(out_written, required);
    }
}

pub(crate) fn mouse_output_needs_space(required: usize, out: *mut u8, out_len: usize) -> bool {
    required > 0 && (out.is_null() || out_len < required)
}
