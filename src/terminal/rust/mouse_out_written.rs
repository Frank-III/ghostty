use core::ptr;

pub(crate) unsafe fn mouse_write_out_written(out_written: *mut usize, value: usize) {
    unsafe {
        ptr::write(out_written, value);
    }
}
