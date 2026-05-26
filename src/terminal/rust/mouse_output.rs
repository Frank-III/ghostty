use crate::mouse_out_written::*;

pub(crate) unsafe fn mouse_suppress_output(out_written: *mut usize) {
    unsafe {
        mouse_write_out_written(out_written, 0);
    }
}
